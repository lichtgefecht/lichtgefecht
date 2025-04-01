#include <stdio.h>
#include "trigger.h"
#include "esp_log.h"


static const char* TAG = "trigger";

void IRAM_ATTR trigger_isr_handler(void* arg) {

    trigger_config_t* cfg = (trigger_config_t*) arg; 
    // Interrupt service routine
    xQueueSendFromISR(cfg->trig_queue, &cfg->gpio_num, NULL);
}

int trigger_create_trigger(trigger_config_t* cfg){
    gpio_config_t io_conf = {
        .intr_type = GPIO_INTR_NEGEDGE,
        .pin_bit_mask = (1ULL << cfg->gpio_num),
        .mode = GPIO_MODE_INPUT,
        // .pull_up_en = GPIO_PULLUP_ENABLE,
        // .pull_down_en = GPIO_PULLDOWN_DISABLE
    };

    gpio_config(&io_conf);

    // Install ISR service
    gpio_install_isr_service(0); // 0 = no flags
    gpio_isr_handler_add(cfg->gpio_num, trigger_isr_handler, (void*)cfg);

    ESP_LOGI(TAG, "Registered trigger on GPIO pin %d\n", cfg->gpio_num);
    
    return 0;
}
