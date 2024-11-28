cmake_minimum_required(VERSION 3.10)
project(cxx)

set(CMAKE_CXX_STANDARD 17)
add_library(cxx STATIC api.cpp)
target_include_directories(cxx PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/../libtorrent/include)
install(TARGETS cxx DESTINATION .)
