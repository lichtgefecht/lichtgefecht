cmake_minimum_required(VERSION 3.16)

# Add the search path for our own components
set(EXTRA_COMPONENT_DIRS $ENV{PROJECT_DIR}../components)
# Explicitly define which components to include in the build
set(COMPONENTS main diag peripherals protobuf-c codec)

# setting variables has to be done before including the project.cmake file, otherwise APPEND may be used
include($ENV{IDF_PATH}/tools/cmake/project.cmake)
project(tagger)

idf_build_set_property(COMPILE_OPTIONS "-Wall" APPEND)
idf_build_set_property(COMPILE_OPTIONS "-Werror" APPEND)
