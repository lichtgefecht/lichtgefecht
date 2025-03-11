
#include "remote.h"
#include <stdio.h>

#include "driver/rmt_rx.h"
#include "driver/rmt_tx.h"
#include "esp_log.h"

// 1MHz resolution, 1 tick = 1us
#define REMOTE_CHANNEL_RESOLUTION_HZ 1000000
// amount of RMT symbols that the channel can store at a time
#define REMOTE_MEM_BLOCK_SYMBOLS 64
// the shortest duration for NEC signal is 560us, 1250ns < 560us, valid signal won't be treated as noise
#define REMOTE_SIGNAL_RANGE_MIN_NS 1250
// the longest duration for NEC signal is 9000us, 12000000ns > 9000us, the receive won't stop early
#define REMOTE_SIGNAL_RANGE_MAX_NS 12000000

static const char* TAG = "remote";

static bool example_rmt_rx_done_callback(rmt_channel_handle_t channel, const rmt_rx_done_event_data_t* edata,
                                         void* user_data) {
    BaseType_t high_task_wakeup = pdFALSE;
    QueueHandle_t receive_queue = (QueueHandle_t)user_data;
    // send the received RMT symbols to the parser task
    xQueueSendFromISR(receive_queue, edata, &high_task_wakeup);
    return high_task_wakeup == pdTRUE;
}

int remote_create_receiver(remote_config_t* cfg) {
    ESP_LOGI(TAG, "create RMT RX channel");
    rmt_rx_channel_config_t rx_channel_cfg = {
        .clk_src = RMT_CLK_SRC_DEFAULT,
        .resolution_hz = REMOTE_CHANNEL_RESOLUTION_HZ,
        .mem_block_symbols = REMOTE_MEM_BLOCK_SYMBOLS,
        .gpio_num = cfg->gpio_num,
    };
    rmt_channel_handle_t rx_channel = NULL;
    ESP_ERROR_CHECK(rmt_new_rx_channel(&rx_channel_cfg, &rx_channel));

    ESP_LOGI(TAG, "register RX done callback");

    rmt_rx_event_callbacks_t cbs = {
        .on_recv_done = example_rmt_rx_done_callback,
    };

    ESP_ERROR_CHECK(rmt_rx_register_event_callbacks(rx_channel, &cbs, cfg->queue));

    // the following timing requirement is based on NEC protocol
    rmt_receive_config_t receive_config = {
        .signal_range_min_ns = REMOTE_SIGNAL_RANGE_MIN_NS,
        .signal_range_max_ns = REMOTE_SIGNAL_RANGE_MAX_NS,
    };

    ESP_ERROR_CHECK(rmt_enable(rx_channel));
    return 0;
}

int remote_create_transmitter(int gpio_num) { return 0; }
