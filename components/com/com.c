#include <stdio.h>
#include <errno.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>

#include "esp_event.h"
#include "esp_log.h"
#include "esp_wifi.h"
#include "esp_netif.h"
#include "lwip/sockets.h"
#include "lwip/netdb.h"
#include "lwip/ip4_addr.h"

#include "codec.h"
#include "com.h"

typedef struct _com_self_s {
    int broadcast_sock;
    struct sockaddr_in broadcast_addr;
    struct sockaddr_in base_addr;
    uint8_t base_address_valid;
    
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

static int _com_get_own_address(uint32_t *address){

    esp_netif_ip_info_t ip_info;
    esp_netif_t *netif = esp_netif_get_handle_from_ifkey("WIFI_STA_DEF");
    if (netif == NULL) {
        ESP_LOGE(TAG, "Failed to get network interface\n");
        return -1;
    }    

    if (esp_netif_get_ip_info(netif, &ip_info) == ESP_OK) {
        *address = ntohl(ip_info.ip.addr);
    } else {
        errno = EINVAL;
        ESP_LOGE(TAG, "Failed to get IP\n");
        return -1;
    }
    
    return 0;
}

static void _com_event_handler(void *arg, esp_event_base_t event_base, int32_t event_id, void *event_data) {
    if (event_base == WIFI_EVENT && event_id == WIFI_EVENT_STA_START) {
        esp_wifi_connect();
    } 
    else if (event_base == WIFI_EVENT && event_id == WIFI_EVENT_STA_STOP) {
        ESP_LOGI(TAG, "Wi-Fi stopped");
        xEventGroupClearBits(s_wifi_event_group, WIFI_CONNECTED_BIT | WIFI_FAIL_BIT);
    } 
    else if (event_base == WIFI_EVENT && event_id == WIFI_EVENT_STA_DISCONNECTED) {
        xEventGroupClearBits(s_wifi_event_group, WIFI_CONNECTED_BIT);

        if (s_retry_num < CONFIG_ESP_MAXIMUM_RETRY) {
            esp_wifi_connect();
            s_retry_num++;
            ESP_LOGI(TAG, "Retrying to connect to the AP (%d/%d)", s_retry_num, CONFIG_ESP_MAXIMUM_RETRY);
        } else {
            s_retry_num = 0; // Reset retry count on final failure
            xEventGroupSetBits(s_wifi_event_group, WIFI_FAIL_BIT);
            ESP_LOGI(TAG, "Failed to connect to the AP");
        }
    } 
    else if (event_base == IP_EVENT && event_id == IP_EVENT_STA_GOT_IP) {
        ip_event_got_ip_t *event = (ip_event_got_ip_t *)event_data;
        ESP_LOGI(TAG, "Connected with IP: " IPSTR, IP2STR(&event->ip_info.ip));
        s_retry_num = 0;
        xEventGroupClearBits(s_wifi_event_group, WIFI_FAIL_BIT);
        xEventGroupSetBits(s_wifi_event_group, WIFI_CONNECTED_BIT);
    }
}

void com_scan_wifi_networks(void) {
    uint16_t num_networks = 10;
    wifi_ap_record_t ap_records[10];

    ESP_LOGW(TAG, "Scanning for networks.....\n");

    ESP_ERROR_CHECK(esp_wifi_scan_start(NULL, true));
    ESP_ERROR_CHECK(esp_wifi_scan_get_ap_records(&num_networks, ap_records));

    for (int i = 0; i < num_networks; i++) {
        ESP_LOGI(TAG, "SSID: %s, RSSI: %d", ap_records[i].ssid, ap_records[i].rssi);
    }
}

void com_init_wifi_station(void) {
    
    esp_log_level_set("wifi", ESP_LOG_VERBOSE);
    
    if (strlen(CONFIG_ESP_WIFI_SSID) == 0 || strlen(CONFIG_ESP_WIFI_PASSWORD) == 0) {
        ESP_LOGE(TAG, "Wi-Fi SSID or Password not set");
        return;
    }

    s_wifi_event_group = xEventGroupCreate();
    if (!s_wifi_event_group) {
        ESP_LOGE(TAG, "Failed to create event group");
        return;
    }

    if (esp_netif_init() != ESP_OK) {
        ESP_LOGE(TAG, "Failed to initialize netif");
        vEventGroupDelete(s_wifi_event_group);
        return;
    }

    if (esp_event_loop_create_default() != ESP_OK) {
        ESP_LOGE(TAG, "Failed to create event loop");
        vEventGroupDelete(s_wifi_event_group);
        return;
    }

    esp_netif_create_default_wifi_sta();
    wifi_init_config_t cfg = WIFI_INIT_CONFIG_DEFAULT();
    if (esp_wifi_init(&cfg) != ESP_OK) {
        ESP_LOGE(TAG, "Failed to initialize Wi-Fi");
        esp_event_loop_delete_default();
        vEventGroupDelete(s_wifi_event_group);
        return;
    }

    esp_event_handler_instance_t instance_any_id;
    esp_event_handler_instance_t instance_got_ip;
    if (esp_event_handler_instance_register(WIFI_EVENT, ESP_EVENT_ANY_ID, &_com_event_handler, NULL, &instance_any_id) != ESP_OK ||
        esp_event_handler_instance_register(IP_EVENT, IP_EVENT_STA_GOT_IP, &_com_event_handler, NULL, &instance_got_ip) != ESP_OK) {
        ESP_LOGE(TAG, "Failed to register event handler");
        esp_wifi_deinit();
        esp_event_loop_delete_default();
        vEventGroupDelete(s_wifi_event_group);
        return;
    }
    
    wifi_config_t wifi_config = {};
    strncpy((char *)wifi_config.sta.ssid, CONFIG_ESP_WIFI_SSID, sizeof(wifi_config.sta.ssid) - 1);
    strncpy((char *)wifi_config.sta.password, CONFIG_ESP_WIFI_PASSWORD, sizeof(wifi_config.sta.password) - 1);
    wifi_config.sta.threshold.authmode = strlen(CONFIG_ESP_WIFI_PASSWORD) < 8 ? WIFI_AUTH_OPEN : WIFI_AUTH_WPA2_PSK;
    
    ESP_ERROR_CHECK(esp_wifi_set_mode(WIFI_MODE_STA));
    ESP_ERROR_CHECK(esp_wifi_set_config(WIFI_IF_STA, &wifi_config));
    ESP_ERROR_CHECK(esp_wifi_start());

    // com_scan_wifi_networks();

    EventBits_t bits = xEventGroupWaitBits(s_wifi_event_group, WIFI_CONNECTED_BIT | WIFI_FAIL_BIT, pdFALSE, pdFALSE, pdMS_TO_TICKS(10000));
    
    if (bits & WIFI_CONNECTED_BIT) {
        ESP_LOGI(TAG, "Connected to SSID:%s", CONFIG_ESP_WIFI_SSID);
    } else {
        ESP_LOGE(TAG, "Connection failed or timed out");
    }

    esp_event_handler_instance_unregister(WIFI_EVENT, ESP_EVENT_ANY_ID, instance_any_id);
    esp_event_handler_instance_unregister(IP_EVENT, IP_EVENT_STA_GOT_IP, instance_got_ip);
    esp_wifi_stop();
    esp_wifi_deinit();
    esp_event_loop_delete_default();
    vEventGroupDelete(s_wifi_event_group);
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

        // codec_print_msg(buffer, bytes_received); // for debugging only
        *msg = lg__msg__unpack(NULL, bytes_received, buffer);
        if(*msg == NULL){
            ESP_LOGE(TAG, "Bad message\n");
            errno = EBADMSG;
            return -1;
        }
    }

    return 0;
}

int com_send_message(const Lg__Msg* msg){

    uint8_t* buf;
    uint32_t len;
    int32_t sent_bytes;

    if(self.base_address_valid == 0){
        ESP_LOGE(TAG, "Trying to send, but base address not known yet.\n");
        errno = EINVAL;
        return -1;
    }

    len = lg__msg__get_packed_size(msg);

    buf = malloc(len);
    lg__msg__pack(msg, buf);

    ESP_LOGI(TAG, "Sending to %lx:%x", self.base_addr.sin_addr.s_addr, self.base_addr.sin_port);

    sent_bytes = sendto(self.broadcast_sock, buf, len , 0, (struct sockaddr *)&self.base_addr, sizeof(self.base_addr));
    
    if (sent_bytes != len){
        ESP_LOGW(TAG, "Tried to send %ld bytes, but %li bytes were sent", len, sent_bytes);
    }

    free(buf);
    return sent_bytes;

}

int com_handle_broadcast(const Lg__Broadcast* msg){
    
    if (msg->reflector_addr_case != LG__BROADCAST__REFLECTOR_ADDR_SOCKET_ADDR)
    {
        ESP_LOGE(TAG, "Address type of broadcast unknown.\n");
        errno = ENOTSUP;
        return -1;
    } 
    
    Lg__SocketAddr *addr = msg->socketaddr;
    switch (addr->ip_case)
    {
    case LG__SOCKET_ADDR__IP_V4:

        self.base_addr.sin_family = AF_INET;
        self.base_addr.sin_port = htons((uint16_t)addr->port);
        self.base_addr.sin_addr.s_addr = htonl(addr->v4);
        self.base_address_valid = 1;
        
        break;
    
    default:
        errno = ENOTSUP;
        return -1;
    }
    return 0;
}


int com_build_broadcast_reply(Lg__BroadcastReply* msg){
    int status;
    
    msg->client_addr_case = LG__BROADCAST_REPLY__CLIENT_ADDR_SOCKET_ADDR;
    msg->socketaddr = malloc(sizeof(Lg__SocketAddr));
    lg__socket_addr__init( msg->socketaddr);
    msg->socketaddr->ip_case = LG__SOCKET_ADDR__IP_V4;
    status = _com_get_own_address(&msg->socketaddr->v4);
    msg->socketaddr->port = 3333;

    return status;
}
