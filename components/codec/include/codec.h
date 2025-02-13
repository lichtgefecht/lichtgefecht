#pragma once
#include <lg.pb-c.h>

#ifdef __cplusplus
extern "C" {
#endif



void codec_print_msg(const unsigned char* buf, int len);


typedef enum {
  BROADCAST = 2,
  UNIMPLEMENTED = 3,
} MsgType;


typedef struct ConInfoIP_s{
  uint32_t addr;
  uint16_t port;
} ConInfoIP;

int codec_parse(const uint8_t* buf, int len, Lg__Msg** msg);

// todo should the codec rely on the concrete protoc type? or is this only an impl detail?
int codec_get_con_info_ip_from_bc(const Lg__Broadcast* broadcast, ConInfoIP* cip);


void* write_bc_reply(int* len);

#ifdef __cplusplus
}
#endif
