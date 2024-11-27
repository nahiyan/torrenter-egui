cmake_minimum_required(VERSION 3.10)
project(cxx)

set(CMAKE_CXX_STANDARD 17)
add_library(cxx STATIC api.cpp)
target_include_directories(cxx PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/../libtorrent/include)
# target_link_directories(cxx PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/../lib)
# target_link_libraries(cxx PRIVATE torrent-rasterbar)
install(TARGETS cxx DESTINATION .)
