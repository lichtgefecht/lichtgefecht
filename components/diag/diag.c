#include <stdio.h>
#include "diag.h"
#include "diag_priv.h"

#include "esp_log.h"
#include "esp_chip_info.h"
#include "esp_flash.h"
#include "esp_log.h"
#include "esp_system.h"

#include "sdkconfig.h"
#include <inttypes.h>

void selftest(const char *app)
{
  ESP_LOGI(TAG_DIAG, "[%s] Startup..", app);
  ESP_LOGI(TAG_DIAG, "[%s] Free memory: %" PRIu32 " bytes", app, esp_get_free_heap_size());
  ESP_LOGI(TAG_DIAG, "[%s] IDF version: %s", app, esp_get_idf_version());

  /* Print chip information */
  esp_chip_info_t chip_info;
  uint32_t flash_size;
  esp_chip_info(&chip_info);
  ESP_LOGI(TAG_DIAG, "This is %s chip with %d CPU core(s), %s%s%s%s, ",
           CONFIG_IDF_TARGET, chip_info.cores,
           (chip_info.features & CHIP_FEATURE_WIFI_BGN) ? "WiFi/" : "",
           (chip_info.features & CHIP_FEATURE_BT) ? "BT" : "",
           (chip_info.features & CHIP_FEATURE_BLE) ? "BLE" : "",
           (chip_info.features & CHIP_FEATURE_IEEE802154) ? ", 802.15.4 (Zigbee/Thread)" : "");

  unsigned major_rev = chip_info.revision / 100;
  unsigned minor_rev = chip_info.revision % 100;
  ESP_LOGI(TAG_DIAG, "Silicon revision v%d.%d", major_rev, minor_rev);
  if (esp_flash_get_size(NULL, &flash_size) != ESP_OK)
  {
    ESP_LOGE(TAG_DIAG, "Get flash size failed");
    return;
  }

  ESP_LOGI(
      TAG_DIAG, "%" PRIu32 "MB %s flash", flash_size / (uint32_t)(1024 * 1024),
      (chip_info.features & CHIP_FEATURE_EMB_FLASH) ? "embedded" : "external");

  ESP_LOGI(TAG_DIAG, "Minimum free heap size: %" PRIu32 " bytes\n",
           esp_get_minimum_free_heap_size());
}