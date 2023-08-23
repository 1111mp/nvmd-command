cmake_minimum_required(VERSION 3.18)

project(nvmd C CXX)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Determine the platform.
if("${CMAKE_SYSTEM_NAME}" STREQUAL "Darwin")
  message("It's OS_MACOSX!")
  add_compile_definitions(NVMD_PLATFORM_MACOS)
elseif("${CMAKE_SYSTEM_NAME}" STREQUAL "Windows")
  add_compile_definitions(NVMD_PLATFORM_WINDOWS)
endif()

file(GLOB_RECURSE ${PROJECT_NAME}_HEADER_FILES src/*.h src/*.hpp)
file(GLOB_RECURSE ${PROJECT_NAME}_SOURCE_FILES src/*.cpp src/*.cc)

add_executable(${PROJECT_NAME}
  ${${PROJECT_NAME}_HEADER_FILES}
  ${${PROJECT_NAME}_SOURCE_FILES}
)

include_directories(
  "include"
)
