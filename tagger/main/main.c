#include <errno.h>
#include <inttypes.h>
#include <lg.pb-c.h>
#include <stdio.h>
#include <string.h>

#include "codec.h"
#include "com.h"
#include "diag.h"
#include "esp_log.h"
#include "esp_system.h"
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "nvs_flash.h"
#include "remote.h"

#define BROADCAST_PRIORITY 5
#define TRIGGER_PRIORITY 5
#define RECEIVER_PRIORITY 5

#define REMOTE_RX_PIN 38
#define REMOTE_TX_PIN 42

remote_config_t rx_cfg;
remote_config_t tx_cfg;

static const char* TAG = "tagger_main";

static void broadcast_handler_task(void* pv_parameters);
static void init_nvs(void);

void app_main(void) {
    selftest("Tagger");

    // init_nvs();
    // com_init_wifi_station();

    ESP_LOGI(TAG, "Wifi ready!\n");

    // xTaskCreate(broadcast_handler_task, "broadcast handler", 4096, NULL, BROADCAST_PRIORITY, NULL);
    // xTaskCreate(trigger_handler_task, "trigger handler", 4096, NULL, TRIGGER_PRIORITY, NULL);

    // Setup the receiver (currently one)
    QueueHandle_t receive_queue = xQueueCreate(1, sizeof(rmt_rx_done_event_data_t));
    memset(&rx_cfg, 0, sizeof(remote_config_t));
    rx_cfg.gpio_num = REMOTE_RX_PIN;
    rx_cfg.queue = receive_queue;
    ESP_ERROR_CHECK(remote_create_receiver(&rx_cfg));

    // Setup the transmitter
    QueueHandle_t transmit_queue = xQueueCreate(1, sizeof(rmt_tx_done_event_data_t));
    memset(&tx_cfg, 0, sizeof(remote_config_t));
    tx_cfg.gpio_num = REMOTE_TX_PIN;
    tx_cfg.queue = transmit_queue;
    ESP_ERROR_CHECK(remote_create_transmitter(&tx_cfg));

    // Start the rx and tx handler tasks
    xTaskCreate(rx_handler_task, "receive handler", 4096, (void*)&rx_cfg, RECEIVER_PRIORITY, NULL);
    xTaskCreate(tx_handler_task, "transmit handler", 4096, (void*)&tx_cfg, RECEIVER_PRIORITY, NULL);

    vTaskSuspend(NULL);
}

static void init_nvs(void) {
    esp_err_t ret = nvs_flash_init();
    if (ret == ESP_ERR_NVS_NO_FREE_PAGES || ret == ESP_ERR_NVS_NEW_VERSION_FOUND) {
        ESP_ERROR_CHECK(nvs_flash_erase());
        ret = nvs_flash_init();
    }
    ESP_ERROR_CHECK(ret);
}

static int build_broadcast_reply(Lg__Msg** built_msg) {
    Lg__Msg* msg = malloc(sizeof(Lg__Msg));
    lg__msg__init(msg);

    msg->hid = "the thing";
    msg->inner_case = LG__MSG__INNER_BROADCAST_REPLY;

    msg->broadcastreply = malloc(sizeof(Lg__BroadcastReply));
    lg__broadcast_reply__init(msg->broadcastreply);
    msg->broadcastreply->devicetype = LG__DEVICE_TYPE__TAGGER;

    com_build_broadcast_reply(msg->broadcastreply);

    *built_msg = msg;

    return 0;
}

static void broadcast_handler_task(void* pv_parameters) {
    (void)pv_parameters;
    Lg__Msg* rcv_msg = NULL;
    Lg__Msg* reply_msg = NULL;
    int status;

    status = com_init();
    if (status != 0) {
        ESP_LOGE(TAG, "Initializing com failed with error %s.\n", strerror(errno));
    }

    while (1) {
        bool broadcast_received = false;
        do {
            status = com_receive_message(&rcv_msg);
            if (status != 0 || rcv_msg == NULL) {
                ESP_LOGE(TAG, "Receiving a broadcast message failed with %s. Deleting self.", strerror(errno));
                continue;
            }

            switch (rcv_msg->inner_case) {
                case LG__MSG__INNER_BROADCAST: {
                    ESP_LOGI(TAG, "Received broadcast\n");
                    status = com_handle_broadcast(rcv_msg->broadcast);
                    if (status == 0) {
                        broadcast_received = true;
                    }
                } break;
                default:
                    ESP_LOGE(TAG, "inner case not defined: %d\n", rcv_msg->inner_case);
                    free(rcv_msg);
                    break;
            }

        } while (!broadcast_received);

        status = build_broadcast_reply(&reply_msg);

        status = com_send_message(reply_msg);
        free(rcv_msg);
        free(reply_msg);
    }
}
