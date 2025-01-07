#include <inttypes.h>
#include <stdio.h>

#include "diag.h"
#include "esp_log.h"
#include "esp_system.h"
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "peripherals.h"
#include "foo/what.pb-c.h"

static const char* TAG = "tagger_main";

void app_main(void) {
    selftest("Tagger");


    AMessage msg = AMESSAGE__INIT;
    AMessage *msg_result;

    Bar bar = BAR__INIT;
    char* baerle = "flauschig";
    // char* baerle = malloc(12);
    
    bar.baerle = baerle;

    void *buf;                     // Buffer to store serialized data
    unsigned len;                  // Length of serialized data

    
    msg.a = 4;
    msg.has_b=1;
    msg.b = 2; 

    msg.bar = &bar;
    msg.inner_case = AMESSAGE__INNER_BAR;
    
    len = amessage__get_packed_size(&msg);
    
    buf = calloc(1, len);
    amessage__pack(&msg,buf);

    for (int i = 0; i<len; i++){
        printf("%02x ",(unsigned int)((unsigned char*)buf)[i]);

    }
    printf("\n");

    fprintf(stderr,"Writing %d serialized bytes\n",len); // See the length of message
    // fwrite(buf,len,1,stderr);

    size_t msg_len = len;
    msg_result = amessage__unpack(NULL, msg_len, buf);	
    if (msg_result == NULL)
    {
        ESP_LOGI(TAG, "unpack fail");
        perror("Failed");
        return;
        // exit(0);
    }

    ESP_LOGI(TAG, "Result a: %ld", msg_result->a);
    ESP_LOGI(TAG, "Result b: %ld", msg_result->b);
    ESP_LOGI(TAG, "Result inner: %d", msg_result->inner_case);
    ESP_LOGI(TAG, "Result baerle: %s", msg_result->bar->baerle);
    ESP_LOGI(TAG, "Ok");




    // init_nvs();
    // wifi_init_station();

    // ESP_LOGI(TAG, "Hello world!\n");

    // for (int i = 10; i >= 0; i--) {
    //     ESP_LOGI(TAG, "Restarting in %d seconds...\n", i);
    //     vTaskDelay(1000 / portTICK_PERIOD_MS);
    // }
    // ESP_LOGI(TAG, "Restarting now.\n");
    // fflush(stdout);
    // esp_restart();
}
