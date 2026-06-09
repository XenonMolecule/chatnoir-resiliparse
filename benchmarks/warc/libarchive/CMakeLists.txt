cmake_minimum_required(VERSION 3.16)
project(libarchive_profile LANGUAGES C)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_C_EXTENSIONS ON)

include(FetchContent)

set(ENABLE_TEST OFF CACHE BOOL "" FORCE)
set(ENABLE_INSTALL OFF CACHE BOOL "" FORCE)
set(ENABLE_TAR OFF CACHE BOOL "" FORCE)
set(ENABLE_CPIO OFF CACHE BOOL "" FORCE)
set(ENABLE_CAT OFF CACHE BOOL "" FORCE)
set(ENABLE_SHARED OFF CACHE BOOL "" FORCE)
set(ENABLE_STATIC ON CACHE BOOL "" FORCE)

FetchContent_Declare(
    libarchive
    GIT_REPOSITORY https://github.com/libarchive/libarchive.git
    GIT_TAG master
    GIT_SHALLOW TRUE
    UPDATE_DISCONNECTED TRUE
)

if(POLICY CMP0169)
    cmake_policy(SET CMP0169 OLD)
endif()

FetchContent_GetProperties(libarchive)
if(NOT libarchive_POPULATED)
    FetchContent_Populate(libarchive)
endif()

set(LIBARCHIVE_WARC_READER
    "${libarchive_SOURCE_DIR}/libarchive/archive_read_support_format_warc.c"
)
file(READ "${LIBARCHIVE_WARC_READER}" LIBARCHIVE_WARC_READER_SOURCE)
string(REPLACE
    "#define HDR_PROBE_LEN\t\t(12U)"
    "#define HDR_PROBE_LEN\t\t(4096U)"
    LIBARCHIVE_WARC_READER_SOURCE
    "${LIBARCHIVE_WARC_READER_SOURCE}"
)
file(WRITE "${LIBARCHIVE_WARC_READER}" "${LIBARCHIVE_WARC_READER_SOURCE}")

add_subdirectory("${libarchive_SOURCE_DIR}" "${libarchive_BINARY_DIR}" EXCLUDE_FROM_ALL)

add_executable(profile profile.c)
target_link_libraries(profile PRIVATE archive_static)
