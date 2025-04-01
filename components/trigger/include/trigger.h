
#pragma once

#include "freertos/FreeRTOS.h"
#include "freertos/queue.h"
//
#include "driver/gpio.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct trigger_config_s {
    gpio_num_t gpio_num;
    QueueHandle_t trig_queue;
} trigger_config_t;

int trigger_create_trigger(trigger_config_t* cfg);

#ifdef __cplusplus
}
#endif

