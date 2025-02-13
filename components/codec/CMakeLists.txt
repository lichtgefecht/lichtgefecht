# if(NOT DEFINED TESTING)
#     include(${CMAKE_CURRENT_SOURCE_DIR}/cmake/embedded/CMakeLists.txt)
# else()
#     include(${CMAKE_CURRENT_SOURCE_DIR}/cmake/test/CMakeLists.txt)
# endif(NOT DEFINED TESTING)


OPTION(TESTING "create build for test" OFF)

if(TESTING)


cmake_minimum_required(VERSION 3.10)
set(IDF_PATH "$ENV{IDF_PATH}")

project(
    codec-tests
    VERSION 0.0.1
    LANGUAGES C)


add_executable(codecTest codecTest.c codec.c ${IDF_PATH}/components/protobuf-c/protobuf-c/protobuf-c/protobuf-c.c)
include_directories(codecTest ${CMAKE_CURRENT_SOURCE_DIR}/include)
include_directories(codecTest ${IDF_PATH}/components/protobuf-c/protobuf-c)


else()
idf_component_register(SRCS "codec.c"
                    INCLUDE_DIRS "include"
                    PRIV_REQUIRES lg_api)
endif(TESTING)
