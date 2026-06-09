cmake_minimum_required(VERSION 3.16)
project(libarchive_profile LANGUAGES C)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_C_EXTENSIONS ON)

include(FetchContent)
find_package(Git REQUIRED)

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

function(apply_libarchive_patch patch_file)
    execute_process(
        COMMAND "${GIT_EXECUTABLE}" apply --check "${patch_file}"
        WORKING_DIRECTORY "${libarchive_SOURCE_DIR}"
        RESULT_VARIABLE patch_check_result
        OUTPUT_QUIET
        ERROR_QUIET
    )
    if(patch_check_result EQUAL 0)
        execute_process(
            COMMAND "${GIT_EXECUTABLE}" apply "${patch_file}"
            WORKING_DIRECTORY "${libarchive_SOURCE_DIR}"
            COMMAND_ERROR_IS_FATAL ANY
        )
    else()
        execute_process(
            COMMAND "${GIT_EXECUTABLE}" apply --reverse --check "${patch_file}"
            WORKING_DIRECTORY "${libarchive_SOURCE_DIR}"
            RESULT_VARIABLE patch_reverse_check_result
            OUTPUT_QUIET
            ERROR_VARIABLE patch_error
        )
        if(NOT patch_reverse_check_result EQUAL 0)
            message(FATAL_ERROR "Cannot apply patch ${patch_file}: ${patch_error}")
        endif()
    endif()
endfunction()

# By default, libarchive skips non-file records, such as requests and metadata.
# Patch it to emit all records.
apply_libarchive_patch("${CMAKE_CURRENT_LIST_DIR}/emit-all-warc-records.patch")

add_subdirectory("${libarchive_SOURCE_DIR}" "${libarchive_BINARY_DIR}" EXCLUDE_FROM_ALL)

add_executable(profile profile.c)
target_link_libraries(profile PRIVATE archive_static)
