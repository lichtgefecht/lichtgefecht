idf_component_register(SRCS "codec.c" "api/what.pb-c.c"
                    INCLUDE_DIRS "include"
                    PRIV_REQUIRES protobuf-c)