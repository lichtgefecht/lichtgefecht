#pragma once

#include <lg.pb-c.h>

#ifdef __cplusplus
extern "C" {
#endif

int com_init(void);
void com_init_wifi_station(void);
int com_get_mac_addr(char* mac);
int com_send_message(const Lg__Msg* msg, const Lg__SocketAddr* to);

int com_receive_message(Lg__Msg** msg);

#ifdef __cplusplus
}
#endif
