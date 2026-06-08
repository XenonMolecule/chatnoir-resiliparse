cmake_minimum_required(VERSION 3.16)
project(warcpp_profile LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

include(FetchContent)

FetchContent_Declare(
    warcpp
    GIT_REPOSITORY https://github.com/pisa-engine/warcpp.git
    GIT_TAG master
)

if(POLICY CMP0169)
    cmake_policy(SET CMP0169 OLD)
endif()

FetchContent_GetProperties(warcpp)
if(NOT warcpp_POPULATED)
    FetchContent_Populate(warcpp)
endif()

add_executable(profile profile.cpp)
target_include_directories(profile PRIVATE
    "${warcpp_SOURCE_DIR}/include"
)
