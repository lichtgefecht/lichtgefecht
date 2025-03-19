#pragma once

#include "driver/rmt_common.h"
#include "remote.h"

#ifdef __cplusplus
extern "C" {
#endif

bool nec_parse_frame(rmt_symbol_word_t* rmt_nec_symbols, remote_scan_code_t* scan_code);

#ifdef __cplusplus
}
#endif
