cmake_minimum_required(VERSION 3.16...3.27)

project(PROJ_NAME LANGUAGES CXX)

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

if(NOT DEFINED CMAKE_CXX_STANDARD)
    set(CMAKE_CXX_STANDARD 17)
endif()

# Rerun:
include(FetchContent)
FetchContent_Declare(rerun_sdk URL https://github.com/rerun-io/rerun/releases/download/0.15.1/rerun_cpp_sdk.zip)
FetchContent_MakeAvailable(rerun_sdk)

add_executable(PROJ_NAME src/main.cpp)
target_link_libraries(PROJ_NAME rerun_sdk)
target_include_directories(PROJ_NAME PRIVATE src)
