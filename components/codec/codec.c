#include "codec.h"

#include <arpa/inet.h>
#include <inttypes.h>
#include <stdio.h>
#include <errno.h>
#include <stdlib.h>
#include "include/codec.h"


static const char* TAG = "codec";

static void codec_print_broadcast(const char* hid, const Lg__Broadcast* broadcast){
    printf("VVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVV\n");
    printf("[%s] Broadcast \n", TAG);
    printf("[%s]     from hid:             %s\n", TAG, hid);
    printf("[%s]     devicetype:           %s\n", TAG, lg__device_type__descriptor.values[broadcast->devicetype].name);
    printf("[%s]     address: \n", TAG);
    if(broadcast->reflector_addr_case == LG__BROADCAST__REFLECTOR_ADDR_SOCKET_ADDR){
        printf("[%s]         ip:               0x%lx\n", TAG, htonl(broadcast->socketaddr->v4));
        printf("[%s]         port:             %ld\n", TAG, broadcast->socketaddr->port);
    }
    printf("^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n");
}

void codec_print_msg(const unsigned char* buf, int len) {
    
    Lg__Msg *msg_result = lg__msg__unpack(NULL, len, buf);

    if(msg_result == NULL){
        // todo
    }

    char* hid = msg_result->hid;

    Lg__Msg__InnerCase inner_case = msg_result->inner_case;
    switch(inner_case){
        case LG__MSG__INNER_BROADCAST:
            codec_print_broadcast(hid, msg_result->broadcast);
            break;
        case LG__MSG__INNER_BROADCAST_REPLY:
            printf("[%s] Received broadcast reply. NOT IMPLEMENTED\n", TAG);
            break;
        case LG__MSG__INNER_TARGET_HIT:
            printf("[%s] Received target hit\n", TAG);
            break;
        default:
            printf("[%s] Unknown MSG received: %d\n", TAG, inner_case); // tut das?
            break;
    }

}
int codec_parse(const uint8_t* const buf, int len, Lg__Msg** msg){
    *msg = lg__msg__unpack(NULL, len, buf);
    if(msg == NULL){
        errno = EINVAL;
        return -1;
    }
    return 0;
}

int codec_get_con_info_ip_from_bc(const Lg__Broadcast* broadcast, ConInfoIP* cip){
    // Lg__Broadcast* bc = (Lg__Broadcast*) broadcast;
    // cip.addr = broadcast->ipaddr->ip;

    // cip.port = bc->ipaddr->port;
    // cip.addr = 0xc0a80092;
    // cip->addr = 0x9200a8c0u;
    cip->addr = htonl(broadcast->socketaddr->v4);
    cip->port = broadcast->socketaddr->port;
    return 0;
}

int codec_write_bc_reply(uint8_t* bytes, int* len){
    struct Lg__BroadcastReply reply = LG__BROADCAST_REPLY__INIT;
    reply.client_addr_case = LG__BROADCAST_REPLY__CLIENT_ADDR_SOCKET_ADDR;
    struct Lg__SocketAddr sockaddr = LG__SOCKET_ADDR__INIT;
    sockaddr.ip_case = LG__SOCKET_ADDR__IP_V4;
    sockaddr.v4 = 1337;
    sockaddr.port = 1337;
    reply.socketaddr = &sockaddr;
    reply.devicetype = LG__DEVICE_TYPE__TAGGER;

    struct Lg__Msg msg = LG__MSG__INIT;
    // msg.inner_case = 
    msg.hid = "the thing";
    msg.broadcastreply = &reply;
    msg.inner_case=LG__MSG__INNER_BROADCAST_REPLY;
    *len = lg__msg__get_packed_size(&msg);
    lg__msg__pack(&msg, bytes);

    return 0;
}