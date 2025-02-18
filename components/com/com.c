#include <stdio.h>
#include <errno.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>

#include "esp_event.h"
#include "esp_log.h"
#include "esp_wifi.h"

#include "codec.h"
#include "com.h"

typedef struct _com_self_s {
    int broadcast_sock;
    struct sockaddr_in broadcast_addr;
} _com_self;

/* FreeRTOS event group to signal when we are connected*/
static EventGroupHandle_t s_wifi_event_group;

static _com_self self;

/* The event group allows multiple bits for each event, but we only care about two events:
 * - we are connected to the AP with an IP
 * - we failed to connect after the maximum amount of retries */
#define WIFI_CONNECTED_BIT BIT0
#define WIFI_FAIL_BIT BIT1

static const char *TAG = "com";

static int s_retry_num = 0;

static void _com_event_handler(void *arg, esp_event_base_t event_base, int32_t event_id, void *event_data) {
    if (event_base == WIFI_EVENT && event_id == WIFI_EVENT_STA_START) {
        esp_wifi_connect();
    } else if (event_base == WIFI_EVENT && event_id == WIFI_EVENT_STA_DISCONNECTED) {

        if (s_retry_num < CONFIG_ESP_MAXIMUM_RETRY) {
            esp_wifi_connect();
            s_retry_num++;
            ESP_LOGI(TAG, "retry to connect to the AP");
        } else {
            xEventGroupSetBits(s_wifi_event_group, WIFI_FAIL_BIT);
        }
        ESP_LOGI(TAG, "connect to the AP fail");
    } else if (event_base == IP_EVENT && event_id == IP_EVENT_STA_GOT_IP) {
        ip_event_got_ip_t *event = (ip_event_got_ip_t *)event_data;
        ESP_LOGI(TAG, "got ip:" IPSTR, IP2STR(&event->ip_info.ip));
        s_retry_num = 0;
        xEventGroupSetBits(s_wifi_event_group, WIFI_CONNECTED_BIT);
    }
}

void com_init_wifi_station(void) {
    s_wifi_event_group = xEventGroupCreate();

    ESP_ERROR_CHECK(esp_netif_init());

    ESP_ERROR_CHECK(esp_event_loop_create_default());
    esp_netif_create_default_wifi_sta();

    wifi_init_config_t cfg = WIFI_INIT_CONFIG_DEFAULT();
    ESP_ERROR_CHECK(esp_wifi_init(&cfg));

    esp_event_handler_instance_t instance_any_id;
    esp_event_handler_instance_t instance_got_ip;
    ESP_ERROR_CHECK(
        esp_event_handler_instance_register(WIFI_EVENT, ESP_EVENT_ANY_ID, &_com_event_handler, NULL, &instance_any_id));
    ESP_ERROR_CHECK(
        esp_event_handler_instance_register(IP_EVENT, IP_EVENT_STA_GOT_IP, &_com_event_handler, NULL, &instance_got_ip));

    wifi_config_t wifi_config = {
        .sta =
            {
                .ssid = CONFIG_ESP_WIFI_SSID,
                .password = CONFIG_ESP_WIFI_PASSWORD,
                /* Authmode threshold resets to WPA2 as default if password matches WPA2 standards (password len => 8).
                 * If you want to connect the device to deprecated WEP/WPA networks, Please set the threshold value
                 * to WIFI_AUTH_WEP/WIFI_AUTH_WPA_PSK and set the password with length and format matching to
                 * WIFI_AUTH_WEP/WIFI_AUTH_WPA_PSK standards.
                 */
                .scan_method = WIFI_FAST_SCAN,
                .sort_method = WIFI_CONNECT_AP_BY_SIGNAL,
                .threshold.rssi = -127,
                .threshold.authmode = WIFI_AUTH_WPA2_PSK,
                // .threshold.authmode = WIFI_AUTH_OPEN,
            },
    };
    ESP_ERROR_CHECK(esp_wifi_set_mode(WIFI_MODE_STA));
    ESP_ERROR_CHECK(esp_wifi_set_config(WIFI_IF_STA, &wifi_config));
    ESP_ERROR_CHECK(esp_wifi_start());

    ESP_LOGI(TAG, "wifi_init_sta finished.");

    /* Waiting until either the connection is established (WIFI_CONNECTED_BIT) or connection failed for the maximum
     * number of re-tries (WIFI_FAIL_BIT). The bits are set by _com_event_handler() (see above) */
    EventBits_t bits =
        xEventGroupWaitBits(s_wifi_event_group, WIFI_CONNECTED_BIT | WIFI_FAIL_BIT, pdFALSE, pdFALSE, portMAX_DELAY);

    /* xEventGroupWaitBits() returns the bits before the call returned, hence we can test which event actually
     * happened. */
    if (bits & WIFI_CONNECTED_BIT) {
        ESP_LOGI(TAG, "connected to ap SSID:%s password:%s", CONFIG_ESP_WIFI_SSID, CONFIG_ESP_WIFI_PASSWORD);
    } else if (bits & WIFI_FAIL_BIT) {
        ESP_LOGI(TAG, "Failed to connect to SSID:%s, password:%s", CONFIG_ESP_WIFI_SSID, CONFIG_ESP_WIFI_PASSWORD);
    } else {
        ESP_LOGE(TAG, "UNPECTED EVENT");
    }
}

// int com_get_mac_addr(uint8_t* mac){
    
//     esp_err_t status;
//     if (mac == NULL){
//         ESP_LOGE(TAG, "mac must not be NULL");
//         return EINVAL;
//     }
//     status = esp_wifi_get_mac(ESP_IF_WIFI_STA, mac);

//     if (status != 0){
//         ESP_LOGE(TAG, "Getting mac address returned esp_err_t 0x%x", status);
//         return EINVAL;
//     }

// }

int com_init(void){

    memset(&self, 0, sizeof(_com_self));

    // Create a UDP socket
    self.broadcast_sock = socket(AF_INET, SOCK_DGRAM, 0);
    if (self.broadcast_sock < 0) {
        ESP_LOGE(TAG, "Socket creation failed\n");
        errno = ENOTSOCK;
        return -1;
    }

    // Configure server address
    memset(&self.broadcast_addr, 0, sizeof(self.broadcast_addr));
    self.broadcast_addr.sin_family = AF_INET;
    self.broadcast_addr.sin_port = htons(3333);
    self.broadcast_addr.sin_addr.s_addr = INADDR_ANY; // Bind to all interfaces

    // Bind the socket to the specified port
    if (bind(self.broadcast_sock, (struct sockaddr *)&self.broadcast_addr, sizeof(self.broadcast_addr)) < 0) {
        ESP_LOGE(TAG, "Binding failed\n");
        close(self.broadcast_sock);
        return -1;
    }

    ESP_LOGI(TAG, "Listening for UDP packets on port 3333\n");
    return 0;
}

int com_receive_message(Lg__Msg** msg){

    fd_set readfds;
    unsigned char buffer[1024];
    socklen_t addr_len;
    int bytes_received;

    FD_ZERO(&readfds);
    FD_SET(self.broadcast_sock, &readfds);

    // Use select to wait for data to be ready
    int activity = select(self.broadcast_sock + 1, &readfds, NULL, NULL, NULL);

    if (activity < 0) {
        ESP_LOGE(TAG, "Select error\n");
        return -1;
    }

    if (FD_ISSET(self.broadcast_sock, &readfds)) {
        // Data is available to read
        addr_len = sizeof(self.broadcast_addr);
        bytes_received = recvfrom(self.broadcast_sock, buffer, sizeof(buffer) - 1, 0,
                                  (struct sockaddr *)&self.broadcast_addr, &addr_len);

        if (bytes_received < 0) {
            ESP_LOGE(TAG, "Receive error\n");
            errno = EBADMSG;
            return -1;
        }

        codec_print_msg(buffer, bytes_received);
        *msg = lg__msg__unpack(NULL, bytes_received, buffer);
        if(*msg == NULL){
            ESP_LOGE(TAG, "Bad message\n");
            errno = EBADMSG;
            return -1;
        }
    }

    return 0;
}

int com_send_message(const Lg__Msg* msg, const Lg__SocketAddr* to){

    uint8_t* buf;
    uint32_t len;

    len = lg__msg__get_packed_size(msg);

    buf = malloc(len);
    lg__msg__pack(msg, buf);

    return sendto(self.broadcast_sock, buf, len , 0, (struct sockaddr *)&self.broadcast_addr, sizeof(self.broadcast_addr));

}
