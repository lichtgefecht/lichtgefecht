/*
 * SPDX-FileCopyrightText: 2021-2022 Espressif Systems (Shanghai) CO LTD
 *
 * SPDX-License-Identifier: Apache-2.0
 */
#pragma once

#include <stdint.h>
#include "driver/rmt_encoder.h"
#include "remote.h"

#ifdef __cplusplus
extern "C" {
#endif


/**
 * @brief Type of IR NEC encoder configuration
 */
typedef struct {
    uint32_t resolution; /*!< Encoder resolution, in Hz */
} remote_encoder_config_t;

/**
 * @brief Create RMT encoder for encoding IR NEC frame into RMT symbols
 *
 * @param[in] config Encoder configuration
 * @param[out] ret_encoder Returned encoder handle
 * @return
 *      - ESP_ERR_INVALID_ARG for any invalid arguments
 *      - ESP_ERR_NO_MEM out of memory when creating IR NEC encoder
 *      - ESP_OK if creating encoder successfully
 */
esp_err_t remote_encoder_new(const remote_encoder_config_t *config, rmt_encoder_handle_t *ret_encoder);

#ifdef __cplusplus
}
#endif
