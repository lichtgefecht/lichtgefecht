#include <inttypes.h>
#include <stdio.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <string.h>
#include <errno.h>

#include "diag.h"
#include "esp_log.h"
#include "esp_system.h"
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "peripherals.h"
#include "codec.h"
#include <lg.pb-c.h>

static const char* TAG = "tagger_main";

static void udp_server_task(void *pvParameters);

void app_main(void) {
    selftest("Tagger");
    // codec_print();

    init_nvs();
    wifi_init_station();

    ESP_LOGI(TAG, "Wifi ready!\n");

    // ESP_ERROR_CHECK(esp_event_loop_create_default());

    xTaskCreate(udp_server_task, "udp_server", 4096, (void*)AF_INET, 5, NULL);

}



static void udp_server_task(void *pvParameters) {
    
    int sock;
    struct sockaddr_in server_addr;
    fd_set readfds;
    unsigned char buffer[1024];
    socklen_t addr_len;
    int bytes_received;

    // Create a UDP socket
    sock = socket(AF_INET, SOCK_DGRAM, 0);
    if (sock < 0) {
        printf("Socket creation failed\n");
        return;
    }

    // Configure server address
    memset(&server_addr, 0, sizeof(server_addr));
    server_addr.sin_family = AF_INET;
    server_addr.sin_port = htons(3333);
    server_addr.sin_addr.s_addr = INADDR_ANY; // Bind to all interfaces

    // Bind the socket to the specified port
    if (bind(sock, (struct sockaddr *)&server_addr, sizeof(server_addr)) < 0) {
        ESP_LOGE(TAG, "Binding failed\n");
        close(sock);
        return;
    }

    ESP_LOGI(TAG, "Listening for UDP packets on port 3333\n");

    // move to another place
    bool initialized = false;
    struct sockaddr_in peer_addr;
    memset(&peer_addr, 0, sizeof(peer_addr));
    peer_addr.sin_family = AF_INET;
    peer_addr.sin_port = htons(3333);
    peer_addr.sin_addr.s_addr = INADDR_ANY;


    while (1) {
        // Clear the set and add the socket to it
        FD_ZERO(&readfds);
        FD_SET(sock, &readfds);

        // Use select to wait for data to be ready
        int activity = select(sock + 1, &readfds, NULL, NULL, NULL);

        if (activity < 0) {
            printf("Select error\n");
            break;
        }

        if (FD_ISSET(sock, &readfds)) {
            // Data is available to read
            addr_len = sizeof(server_addr);
            bytes_received = recvfrom(sock, buffer, sizeof(buffer) - 1, 0,
                                      (struct sockaddr *)&server_addr, &addr_len);

            if (bytes_received < 0) {
                ESP_LOGE(TAG, "Receive error\n");
                break;
            }

            // buffer[bytes_received] = '\0';
            codec_print_msg(buffer, bytes_received);

            Lg__Msg* msg;
            int err;
            if((err = codec_parse(buffer, bytes_received, &msg)) != 0)
            {
                ESP_LOGE(TAG, "Parsing failed with %s\n", strerror(err));
            }
            switch (msg->inner_case) {
                case LG__MSG__INNER_BROADCAST: {
                    // should go somewhere like handle_broadcast
                    if(!initialized){
                        //todo re-enable
                        // initialized = true;
                        ESP_LOGI(TAG, "Initializing with received broadcast\n");

                        ConInfoIP cip;
                        codec_get_con_info_ip_from_bc(msg->broadcast, &cip);

                        peer_addr.sin_port= htons(cip.port);
                        ESP_LOGI(TAG, "ip is 0x%lx\n", cip.addr);
                        peer_addr.sin_addr.s_addr = cip.addr;

                        int len = 0;
                        void* ptr = write_bc_reply(&len);
                        int err = sendto(sock, ptr, len , 0, (struct sockaddr *)&peer_addr, sizeof(peer_addr));
                        ESP_LOGI(TAG, "send summary err=%d, len=%d\n", err, len);

                    }
                    break;
                }
                default:
                    ESP_LOGE(TAG, "inner case not defined: %d\n", msg->inner_case);
                    break;
            }

            // Null-terminate the received data and print it
            // ESP_LOGI(TAG, "Received: %s\n", buffer);
        }
    }

    // Clean up
    close(sock);
    ESP_LOGE(TAG, "Socket closed\n");
}


// {
//     char rx_buffer[128];
//     char addr_str[128];
//     int addr_family = (int)pvParameters;
//     int ip_protocol = 0;
//     struct sockaddr_in6 dest_addr;

//     while (1) {

//         if (addr_family == AF_INET) {
//             struct sockaddr_in *dest_addr_ip4 = (struct sockaddr_in *)&dest_addr;
//             dest_addr_ip4->sin_addr.s_addr = htonl(INADDR_ANY);
//             dest_addr_ip4->sin_family = AF_INET;
//             dest_addr_ip4->sin_port = htons(PORT);
//             ip_protocol = IPPROTO_IP;
//         } else if (addr_family == AF_INET6) {
//             bzero(&dest_addr.sin6_addr.un, sizeof(dest_addr.sin6_addr.un));
//             dest_addr.sin6_family = AF_INET6;
//             dest_addr.sin6_port = htons(PORT);
//             ip_protocol = IPPROTO_IPV6;
//         }

//         int sock = socket(addr_family, SOCK_DGRAM, ip_protocol);
//         if (sock < 0) {
//             ESP_LOGE(TAG, "Unable to create socket: errno %d", errno);
//             break;
//         }
//         ESP_LOGI(TAG, "Socket created");

//         // Set timeout
//         struct timeval timeout;
//         timeout.tv_sec = 10;
//         timeout.tv_usec = 0;
//         setsockopt (sock, SOL_SOCKET, SO_RCVTIMEO, &timeout, sizeof timeout);

//         int err = bind(sock, (struct sockaddr *)&dest_addr, sizeof(dest_addr));
//         if (err < 0) {
//             ESP_LOGE(TAG, "Socket unable to bind: errno %d", errno);
//         }
//         ESP_LOGI(TAG, "Socket bound, port %d", PORT);

//         struct sockaddr_storage source_addr; // Large enough for both IPv4 or IPv6
//         socklen_t socklen = sizeof(source_addr);


//         while (1) {
//             ESP_LOGI(TAG, "Waiting for data");

//             int len = recvfrom(sock, rx_buffer, sizeof(rx_buffer) - 1, 0, (struct sockaddr *)&source_addr, &socklen);

//             // Error occurred during receiving
//             if (len < 0) {
//                 ESP_LOGE(TAG, "recvfrom failed: errno %d", errno);
//                 break;
//             }
//             // Data received
//             else {
//                 // Get the sender's ip address as string
//                 if (source_addr.ss_family == PF_INET) {
//                     inet_ntoa_r(((struct sockaddr_in *)&source_addr)->sin_addr, addr_str, sizeof(addr_str) - 1);
//                 } else if (source_addr.ss_family == PF_INET6) {
//                     inet6_ntoa_r(((struct sockaddr_in6 *)&source_addr)->sin6_addr, addr_str, sizeof(addr_str) - 1);
//                 }

//                 rx_buffer[len] = 0; // Null-terminate whatever we received and treat like a string...
//                 ESP_LOGI(TAG, "Received %d bytes from %s:", len, addr_str);
//                 ESP_LOGI(TAG, "%s", rx_buffer);

//                 int err = sendto(sock, rx_buffer, len, 0, (struct sockaddr *)&source_addr, sizeof(source_addr));
//                 if (err < 0) {
//                     ESP_LOGE(TAG, "Error occurred during sending: errno %d", errno);
//                     break;
//                 }
//             }
//         }

//         if (sock != -1) {
//             ESP_LOGE(TAG, "Shutting down socket and restarting...");
//             shutdown(sock, 0);
//             close(sock);
//         }
//     }
//     vTaskDelete(NULL);
// }