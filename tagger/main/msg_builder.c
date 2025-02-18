
#include <errno.h>

#include "com.h"
#include "codec.h"
#include "lg.pb-c.h"
#include "msg_builder.h"

// int msg_build_bc_reply_msg(uint8_t* msg, int* len)
// {
//     struct Lg__BroadcastReply reply = LG__BROADCAST_REPLY__INIT;
//     reply.client_addr_case = LG__BROADCAST_REPLY__CLIENT_ADDR_SOCKET_ADDR;
//     struct Lg__SocketAddr sockaddr = LG__SOCKET_ADDR__INIT;
//     sockaddr.ip_case = LG__SOCKET_ADDR__IP_V4;
//     sockaddr.v4 = 1337;
//     sockaddr.port = 1337;
//     reply.socketaddr = &sockaddr;
//     reply.devicetype = LG__DEVICE_TYPE__TAGGER;

//     struct Lg__Msg msg = LG__MSG__INIT;
//     msg.hid = "the thing";
//     msg.broadcastreply = &reply;
//     msg.inner_case=LG__MSG__INNER_BROADCAST_REPLY;
//     *len = lg__msg__get_packed_size(&msg);
//     lg__msg__pack(&msg, bytes);

//     return 0;
// }