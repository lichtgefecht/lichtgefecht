#pragma once
#include "freertos/FreeRTOS.h"
//
#include "driver/rmt_common.h"
#include "freertos/queue.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct remote_config_s {
    gpio_num_t gpio_num;
    QueueHandle_t queue;
} remote_config_t;

int remote_create_receiver(remote_config_t* cfg);
int remote_create_transmitter(int gpio_num);

#ifdef __cplusplus
}
#endif
