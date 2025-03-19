idf_component_register(SRCS "remote.c" "ir_nec_encoder.c"
                    INCLUDE_DIRS "include"
                    PRIV_REQUIRES esp_driver_rmt freertos)
