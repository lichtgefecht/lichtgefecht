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
#include "trigger.h"

#define BROADCAST_PRIORITY 5
#define TRIGGER_PRIORITY 5
#define RECEIVER_PRIORITY 5
#define MAIN_PRIORITY 5

#define TRIGGER_PIN 21
#define REMOTE_RX_PIN 38
#define REMOTE_TX_PIN 42

remote_config_t rx_cfg;
remote_config_t tx_cfg;
trigger_config_t trig_cfg;

static const char* TAG = "tagger_main";

static void broadcast_handler_task(void* pv_parameters);
static void init_nvs(void);
static void main_loop_task(void* pv_parameters);
static int hit(remote_scan_code_t* hit_msg);

#define RECEIVE_QUEUE_LENGTH 1
#define TRANSMIT_QUEUE_LENGTH 1
#define COMBINED_LENGTH ( RECEIVE_QUEUE_LENGTH + \
                          TRANSMIT_QUEUE_LENGTH )

typedef struct main_loop_config_s{
    QueueHandle_t receive_queue;
    QueueHandle_t transmit_queue;
    QueueHandle_t trigger_queue;
    QueueSetHandle_t xQueueSet;
}main_loop_config_t;


void app_main(void) {
    selftest("Tagger");

    init_nvs();
    com_init_wifi_station();

    ESP_LOGI(TAG, "Wifi ready!\n");

    // Setup the receiver (currently one)
    QueueHandle_t receive_queue_raw = xQueueCreate(1, sizeof(rmt_rx_done_event_data_t));
    QueueHandle_t receive_queue = xQueueCreate(RECEIVE_QUEUE_LENGTH, sizeof(rmt_rx_done_event_data_t));
    memset(&rx_cfg, 0, sizeof(remote_config_t));
    rx_cfg.gpio_num = REMOTE_RX_PIN;
    rx_cfg.raw_queue = receive_queue_raw;
    rx_cfg.encoded_queue = receive_queue;
    ESP_ERROR_CHECK(remote_create_receiver(&rx_cfg));

    // Setup the transmitter
    QueueHandle_t transmit_queue = xQueueCreate(TRANSMIT_QUEUE_LENGTH, sizeof(gpio_num_t));
    memset(&tx_cfg, 0, sizeof(remote_config_t));
    tx_cfg.gpio_num = REMOTE_TX_PIN;
    tx_cfg.encoded_queue = transmit_queue;
    ESP_ERROR_CHECK(remote_create_transmitter(&tx_cfg));

    // Setup the trigger
    QueueHandle_t trigger_queue = xQueueCreate(1, sizeof(gpio_num_t));
    memset(&trig_cfg, 0, sizeof(trigger_config_t));
    trig_cfg.gpio_num = TRIGGER_PIN;
    trig_cfg.trig_queue = trigger_queue;
    ESP_ERROR_CHECK(trigger_create_trigger(&trig_cfg));

    QueueSetHandle_t xQueueSet = xQueueCreateSet( COMBINED_LENGTH );
    xQueueAddToSet( receive_queue, xQueueSet );
    xQueueAddToSet( trigger_queue, xQueueSet );

    // Start the rx and tx handler tasks
    xTaskCreate(rx_handler_task, "receive handler", 4096, (void*)&rx_cfg, RECEIVER_PRIORITY, NULL);
    xTaskCreate(tx_handler_task, "transmit handler", 4096, (void*)&tx_cfg, RECEIVER_PRIORITY, NULL);
    
    // Start the broadcast handling
    xTaskCreate(broadcast_handler_task, "broadcast handler", 4096, NULL, BROADCAST_PRIORITY, NULL);

    main_loop_config_t cfg = {
        .xQueueSet = xQueueSet,
        .receive_queue = receive_queue,
        .transmit_queue = transmit_queue,
        .trigger_queue = trigger_queue,
    };
    xTaskCreate(main_loop_task, "main loop", 4096, (void*)&cfg, MAIN_PRIORITY, NULL);

    vTaskSuspend(NULL);
}

static void main_loop_task(void* pv_parameters) {

    main_loop_config_t* cfg = (main_loop_config_t*) pv_parameters;    
    remote_scan_code_t hit_msg;
    gpio_num_t trig;

    QueueSetMemberHandle_t xActivatedMember;
    while (true)
    {
        xActivatedMember = xQueueSelectFromSet( cfg->xQueueSet, portMAX_DELAY);
        if (xActivatedMember == cfg->receive_queue)
        {
            xQueueReceive(xActivatedMember, &hit_msg, 0 );
            hit(&hit_msg);
        }
        else if( xActivatedMember == cfg->trigger_queue){
            xQueueReceive(xActivatedMember, &trig, 0 );
            ESP_LOGI(TAG, "Received a trigger from pin %d\n", trig);
            xQueueSend(cfg->transmit_queue, &trig, 0);
        }
        
        else{
            ESP_LOGW(TAG, "Wrong event\n");
            // huh? 
        }
    }
}

static int hit(remote_scan_code_t* hit_msg){
    
    int status;
    Lg__Msg* msg = malloc(sizeof(Lg__Msg));
    lg__msg__init(msg);
    msg->hid = "the thing";

    msg->inner_case = LG__MSG__INNER_TARGET_HIT;
    msg->targethit = malloc(sizeof(Lg__TargetHit));

    lg__target_hit__init(msg->targethit);

    msg->targethit->fromid = hit_msg->address << 16 | hit_msg->command;
    
    status = com_send_message(msg);
    if (status < 0){
        ESP_LOGE(TAG, "Sending target hit message failed with error %s (%d)\n", strerror(errno), errno);
    } else {
        ESP_LOGI(TAG, "Sent %i bytes\n", status);
    }

    free(msg->targethit);
    free(msg);

    return 0;
}

static void init_nvs(void) {
    esp_err_t ret = nvs_flash_init();
    if (ret == ESP_ERR_NVS_NO_FREE_PAGES || ret == ESP_ERR_NVS_NEW_VERSION_FOUND) {
        ESP_LOGI(TAG, "Erasing flash\n");
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
