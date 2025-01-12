#include "codec.h"

#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include "api/what.pb-c.h"
#include "include/codec.h"


int main() {


    void *buf;
    unsigned len;


    Lichtgefecht__Foo foo = LICHTGEFECHT__FOO__INIT;
    foo.foole = 42;
    len = lichtgefecht__foo__get_packed_size(&foo);
    buf = calloc(1, len);
    int result = lichtgefecht__foo__pack(&foo, buf);
    if (result == 0) {
        return -1;
    }
    


    FILE *fptr;
    fptr = fopen("test.bin", "w");
    fwrite(buf, len, 1, fptr);
    fclose(fptr); 
    codec_print();
}