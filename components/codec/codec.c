#include "codec.h"

#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include "api/what.pb-c.h"


static const char* TAG = "codec";



void codec_print() {
    Lichtgefecht__AMessage msg = LICHTGEFECHT__AMESSAGE__INIT;
    Lichtgefecht__AMessage *msg_result;

    Lichtgefecht__Bar bar = LICHTGEFECHT__BAR__INIT;
    char* baerle = "flauschig";
    // char* baerle = malloc(12);
    
    bar.baerle = baerle;

    void *buf;                     // Buffer to store serialized data
    unsigned len;                  // Length of serialized data

    
    msg.a = 4;
    msg.has_b=1;
    msg.b = 2; 

    msg.bar = &bar;
    msg.inner_case = LICHTGEFECHT__AMESSAGE__INNER_BAR;
    
    len = lichtgefecht__amessage__get_packed_size(&msg);
    
    buf = calloc(1, len);
    lichtgefecht__amessage__pack(&msg,buf);

    for (int i = 0; i<len; i++){
        printf("%02x ",(unsigned int)((unsigned char*)buf)[i]);

    }
    printf("\n");

    fprintf(stderr,"Writing %d serialized bytes\n",len); // See the length of message
    // fwrite(buf,len,1,stderr);

    size_t msg_len = len;
    msg_result = lichtgefecht__amessage__unpack(NULL, msg_len, buf);	
    if (msg_result == NULL)
    {
        printf("unpack fail");
        perror("Failed");
        return;
        // exit(0);
    }

    printf("[%s] Result a: %ld", TAG,  msg_result->a);
    // printf(TAG, "Result b: %ld", msg_result->b);
    // printf(TAG, "Result inner: %d", msg_result->inner_case);
    // printf(TAG, "Result baerle: %s", msg_result->bar->baerle);
    // printf(TAG, "Ok");

}