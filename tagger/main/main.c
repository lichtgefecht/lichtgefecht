#include <inttypes.h>
#include <stdio.h>

#include "diag.h"
#include "esp_log.h"
#include "esp_system.h"
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "peripherals.h"

static const char* TAG = "tagger_main";

void app_main(void) {
    selftest("Tagger");

    init_nvs();
    wifi_init_station();

    ESP_LOGI(TAG, "Hello world!\n");

    for (int i = 10; i >= 0; i--) {
        ESP_LOGI(TAG, "Restarting in %d seconds...\n", i);
        vTaskDelay(1000 / portTICK_PERIOD_MS);
    }
    ESP_LOGI(TAG, "Restarting now.\n");
    fflush(stdout);
    esp_restart();
}
