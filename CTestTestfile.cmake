# CMake generated Testfile for 
# Source directory: C:/Users/jthec/ninja
# Build directory: C:/Users/jthec/ninja
# 
# This file includes the relevant testing commands required for 
# testing this directory and lists subdirectories to be tested as well.
if(CTEST_CONFIGURATION_TYPE MATCHES "^([Dd][Ee][Bb][Uu][Gg])$")
  add_test(NinjaTest "C:/Users/jthec/ninja/Debug/ninja_test.exe")
  set_tests_properties(NinjaTest PROPERTIES  _BACKTRACE_TRIPLES "C:/Users/jthec/ninja/CMakeLists.txt;332;add_test;C:/Users/jthec/ninja/CMakeLists.txt;0;")
elseif(CTEST_CONFIGURATION_TYPE MATCHES "^([Rr][Ee][Ll][Ee][Aa][Ss][Ee])$")
  add_test(NinjaTest "C:/Users/jthec/ninja/Release/ninja_test.exe")
  set_tests_properties(NinjaTest PROPERTIES  _BACKTRACE_TRIPLES "C:/Users/jthec/ninja/CMakeLists.txt;332;add_test;C:/Users/jthec/ninja/CMakeLists.txt;0;")
elseif(CTEST_CONFIGURATION_TYPE MATCHES "^([Mm][Ii][Nn][Ss][Ii][Zz][Ee][Rr][Ee][Ll])$")
  add_test(NinjaTest "C:/Users/jthec/ninja/MinSizeRel/ninja_test.exe")
  set_tests_properties(NinjaTest PROPERTIES  _BACKTRACE_TRIPLES "C:/Users/jthec/ninja/CMakeLists.txt;332;add_test;C:/Users/jthec/ninja/CMakeLists.txt;0;")
elseif(CTEST_CONFIGURATION_TYPE MATCHES "^([Rr][Ee][Ll][Ww][Ii][Tt][Hh][Dd][Ee][Bb][Ii][Nn][Ff][Oo])$")
  add_test(NinjaTest "C:/Users/jthec/ninja/RelWithDebInfo/ninja_test.exe")
  set_tests_properties(NinjaTest PROPERTIES  _BACKTRACE_TRIPLES "C:/Users/jthec/ninja/CMakeLists.txt;332;add_test;C:/Users/jthec/ninja/CMakeLists.txt;0;")
else()
  add_test(NinjaTest NOT_AVAILABLE)
endif()
subdirs("_deps/googletest-build")
