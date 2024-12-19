#include "esp_log.h"
#include "esp_system.h"
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include <inttypes.h>
#include <stdio.h>
#include "diag.h"
#include "tagger.h"

void app_main(void) {
    
  selftest("Tagger");

  ESP_LOGI(TAGGER_TAG, "Hello world!\n");

  for (int i = 10; i >= 0; i--) {
    ESP_LOGI(TAGGER_TAG, "Restarting in %d seconds...\n", i);
    vTaskDelay(1000 / portTICK_PERIOD_MS);
  }
  ESP_LOGI(TAGGER_TAG, "Restarting now.\n");
  fflush(stdout);
  esp_restart();
}
