# Loaded immediately after the pinned native project's project() call. Wait
# until its first-party dependencies exist before adding the binding target.
cmake_minimum_required(VERSION 3.19)
set(ZIRCON_VM_LINUX_BINDING_RECIPE "${CMAKE_CURRENT_LIST_DIR}/native-linux.cmake")
cmake_language(DEFER CALL include "${ZIRCON_VM_LINUX_BINDING_RECIPE}")
