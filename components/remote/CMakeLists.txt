idf_component_register(SRCS "remote.c" "remote_encoder.c" "remote_decoder.c"
                    INCLUDE_DIRS "include"
                    PRIV_REQUIRES esp_driver_rmt freertos)
