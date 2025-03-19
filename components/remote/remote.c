
#include "remote.h"

#include <stdio.h>

#include "driver/rmt_rx.h"
#include "driver/rmt_tx.h"
#include "esp_log.h"
#include "remote_decoder.h"
#include "remote_encoder.h"

// 1MHz resolution, 1 tick = 1us
#define REMOTE_CHANNEL_RESOLUTION_HZ 1000000
// amount of RMT symbols that the channel can store at a time
#define REMOTE_MEM_BLOCK_SYMBOLS 64
// the shortest duration for NEC signal is 560us, 1250ns < 560us, valid signal won't be treated as noise
#define REMOTE_SIGNAL_RANGE_MIN_NS 1250
// the longest duration for NEC signal is 9000us, 12000000ns > 9000us, the receive won't stop early
#define REMOTE_SIGNAL_RANGE_MAX_NS 12000000
#define REMOTE_TX_QUEUE_DEPTH 4

static const char* TAG = "remote";

static const rmt_receive_config_t receive_config = {
    .signal_range_min_ns = REMOTE_SIGNAL_RANGE_MIN_NS,
    .signal_range_max_ns = REMOTE_SIGNAL_RANGE_MAX_NS,
};

// this example won't send NEC frames in a loop
const rmt_transmit_config_t transmit_config = {
    .loop_count = 0,  // no loop
};

static const rmt_carrier_config_t carrier_cfg = {
    .duty_cycle = 0.33,
    .frequency_hz = 38000,  // 38KHz
};

static bool example_rmt_rx_done_callback(rmt_channel_handle_t channel, const rmt_rx_done_event_data_t* edata,
                                         void* user_data) {
    BaseType_t high_task_wakeup = pdFALSE;
    QueueHandle_t receive_queue = (QueueHandle_t)user_data;
    // send the received RMT symbols to the parser task
    xQueueSendFromISR(receive_queue, edata, &high_task_wakeup);
    return high_task_wakeup == pdTRUE;
}

/**
 * @brief Decode RMT symbols into NEC scan code and print the result
 */
static void remote_handle_nec_frame(rmt_symbol_word_t* rmt_nec_symbols, size_t symbol_num) {
    // printf("NEC frame start---\r\n");
    // for (size_t i = 0; i < symbol_num; i++) {
    //     printf("{%d:%d},{%d:%d}\r\n", rmt_nec_symbols[i].level0, rmt_nec_symbols[i].duration0,
    //            rmt_nec_symbols[i].level1, rmt_nec_symbols[i].duration1);
    // }
    // printf("---NEC frame end: ");

    if (symbol_num != 34) {
        ESP_LOGE(TAG, "Unknown NEC frame\r\n\r\n");
        return;
    }

    remote_scan_code_t scan_code = {
        .address = 0,
        .command = 0,
    };

    if (!nec_parse_frame(rmt_nec_symbols, &scan_code)) {
        ESP_LOGE(TAG, "Frame could not be parsed");
    }

    printf("Address=%04X, Command=%04X\r\n\r\n", scan_code.address, scan_code.command);
}

int remote_create_receiver(remote_config_t* cfg) {
    ESP_LOGI(TAG, "create RMT RX channel");
    rmt_rx_channel_config_t rx_channel_cfg = {
        .clk_src = RMT_CLK_SRC_DEFAULT,
        .resolution_hz = REMOTE_CHANNEL_RESOLUTION_HZ,
        .mem_block_symbols = REMOTE_MEM_BLOCK_SYMBOLS,
        .gpio_num = cfg->gpio_num,
    };
    cfg->channel = NULL;
    ESP_ERROR_CHECK(rmt_new_rx_channel(&rx_channel_cfg, &cfg->channel));

    ESP_LOGI(TAG, "register RX done callback");

    rmt_rx_event_callbacks_t cbs = {
        .on_recv_done = example_rmt_rx_done_callback,
    };

    ESP_ERROR_CHECK(rmt_rx_register_event_callbacks(cfg->channel, &cbs, cfg->queue));

    ESP_ERROR_CHECK(rmt_enable(cfg->channel));
    return 0;
}

int remote_create_transmitter(remote_config_t* cfg) {
    ESP_LOGI(TAG, "create RMT TX channel");
    rmt_tx_channel_config_t tx_channel_cfg = {
        .clk_src = RMT_CLK_SRC_DEFAULT,
        .resolution_hz = REMOTE_CHANNEL_RESOLUTION_HZ,
        .mem_block_symbols = REMOTE_MEM_BLOCK_SYMBOLS,
        .trans_queue_depth = REMOTE_TX_QUEUE_DEPTH,
        .gpio_num = cfg->gpio_num,
    };

    cfg->channel = NULL;
    ESP_ERROR_CHECK(rmt_new_tx_channel(&tx_channel_cfg, &cfg->channel));

    ESP_LOGI(TAG, "modulate carrier to TX channel");
    ESP_ERROR_CHECK(rmt_apply_carrier(cfg->channel, &carrier_cfg));

    ESP_LOGI(TAG, "enable RMT TX channel");
    ESP_ERROR_CHECK(rmt_enable(cfg->channel));
    return 0;
}

void rx_handler_task(void* pv_parameters) {
    remote_config_t* rmt_cfg = (remote_config_t*)pv_parameters;

    // filled from the queue, supplied by the ISR (done_callback)
    rmt_rx_done_event_data_t rx_data;

    // high-low pairs
    rmt_symbol_word_t raw_symbols[64];

    while (1) {
        // start receive job, this will enable the done_callback
        // the done_callback will then write the data into the queue
        ESP_ERROR_CHECK(rmt_receive(rmt_cfg->channel, raw_symbols, sizeof(raw_symbols), &receive_config));
        if (xQueueReceive(rmt_cfg->queue, &rx_data, portMAX_DELAY) == pdPASS) {
            remote_handle_nec_frame(rx_data.received_symbols, rx_data.num_symbols);
        } else {
            ESP_LOGE(TAG,
                     "rx_handler_task did not receive element in the queue within portMAX_DELAY(%ld)ms, giving up.",
                     portMAX_DELAY);
            vTaskDelete(NULL);
        }
    }
}

void tx_handler_task(void* pv_parameters) {
    remote_config_t* rmt_cfg = (remote_config_t*)pv_parameters;

    ESP_LOGI(TAG, "Install IR NEC encoder");
    remote_encoder_config_t nec_encoder_cfg = {
        .resolution = REMOTE_CHANNEL_RESOLUTION_HZ,
    };
    rmt_encoder_handle_t nec_encoder = NULL;
    ESP_ERROR_CHECK(remote_encoder_new(&nec_encoder_cfg, &nec_encoder));

    while (!0) {
        vTaskDelay(100);

        const remote_scan_code_t scan_code = {
            .address = 0x0440,
            .command = 0x3003,
        };

        ESP_ERROR_CHECK(rmt_transmit(rmt_cfg->channel, nec_encoder, &scan_code, sizeof(scan_code), &transmit_config));
    }
}
