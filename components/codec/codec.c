#include "codec.h"

#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include "build/api/lg.pb-c.h"


static const char* TAG = "codec";

static void codec_print_broadcast(const char* hid, const Lg__Broadcast* broadcast){
    printf("VVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVV\n");
    printf("[%s] Broadcast \n", TAG);
    printf("[%s]     from hid:             %s\n", TAG, hid);
    printf("[%s]     transport layer type: %s\n", TAG, lg__transport_layer__descriptor.values[broadcast->transportlayer].name);
    printf("[%s]     devicetype:           %s\n", TAG, lg__device_type__descriptor.values[broadcast->devicetype].name);
    printf("[%s]     address: \n", TAG);
    if(broadcast->reflector_addr_case == LG__BROADCAST__REFLECTOR_ADDR_IP_ADDR){
        printf("[%s]         ip:               %s\n", TAG, broadcast->ipaddr->ip);
        printf("[%s]         port:             %ld\n", TAG, broadcast->ipaddr->port);
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



    // Lg__AMessage msg = LG__AMESSAGE__INIT;
    // Lg__AMessage *msg_result;

    // Lg__Bar bar =LG__BAR__INIT;
    // char* baerle = "flauschig";
    // // char* baerle = malloc(12);
    
    // bar.baerle = baerle;

    // void *buf;                     // Buffer to store serialized data
    // unsigned len;                  // Length of serialized data

    
    // msg.a = 4;
    // msg.has_b=1;
    // msg.b = 2; 

    // msg.bar = &bar;
    // msg.inner_case = LG__AMESSAGE__INNER_BAR;
    
    // len = lg__amessage__get_packed_size(&msg);
    
    // buf = calloc(1, len);
    // lg__amessage__pack(&msg,buf);

    // for (int i = 0; i<len; i++){
    //     printf("%02x ",(unsigned int)((unsigned char*)buf)[i]);

    // }
    // printf("\n");

    // fprintf(stderr,"Writing %d serialized bytes\n",len); // See the length of message
    // // fwrite(buf,len,1,stderr);

    // size_t msg_len = len;
    // msg_result = lg__amessage__unpack(NULL, msg_len, buf);	
    // if (msg_result == NULL)
    // {
    //     printf("unpack fail");
    //     perror("Failed");
    //     return;
    //     // exit(0);
    // }

    // printf("[%s] Result a: %ld", TAG,  msg_result->a);
    // // printf(TAG, "Result b: %ld", msg_result->b);
    // // printf(TAG, "Result inner: %d", msg_result->inner_case);
    // // printf(TAG, "Result baerle: %s", msg_result->bar->baerle);
    // // printf(TAG, "Ok");

}