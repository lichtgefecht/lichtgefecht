#pragma once
#include "freertos/FreeRTOS.h"
//
#include "driver/rmt_common.h"
#include "freertos/queue.h"

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @brief IR NEC scan code representation
 */
typedef struct {
    uint16_t address;
    uint16_t command;
} remote_scan_code_t;

typedef struct remote_config_s {
    gpio_num_t gpio_num;
    QueueHandle_t raw_queue;
    QueueHandle_t encoded_queue;
    rmt_channel_handle_t channel;
} remote_config_t;

int remote_create_receiver(remote_config_t* cfg);
int remote_create_transmitter(remote_config_t* cfg);

void rx_handler_task(void* pv_parameters);
void tx_handler_task(void* pv_parameters);

#ifdef __cplusplus
}
#endif
