set_languages("c23", "c++23")
add_requires("sentencepiece v0.2.1", {configs = {shared = true}})
add_requires("ctranslate2 v4.6.2", {configs = {shared = true}})
add_requires("boost 1.90.0", {configs = {shared = true}})
add_requires("re2 2025.11.05", {configs = {shared = true}})
add_requires("jnipp v1.0.0")
add_requires("openmp")
add_rules("mode.debug", "mode.release")

-- We need to re-enable shared libraries, hence why we do this here.
package("sentencepiece")
    set_homepage("https://github.com/google/sentencepiece")
    set_description("Unsupervised text tokenizer for Neural Network-based text generation. .")
    set_license("Apache-2.0")

    add_urls("https://github.com/google/sentencepiece/archive/$(version).tar.gz",
             "https://github.com/google/sentencepiece.git")

    add_versions("v0.1.97", "41c3a07f315e3ac87605460c8bb8d739955bc8e7f478caec4017ef9b7d78669b")

    add_deps("cmake", "gperftools")

    add_configs("external_abseil",  {description = "Use external abseil.", default = false, type = "boolean"})
    add_configs("builtin_protobuf", {description = "Use built-in protobuf.", default = true, type = "boolean"})
    if is_plat("windows") then
        add_configs("shared",     {description = "Build shared library.", default = false, type = "boolean"})
        add_configs("vs_runtime", {description = "Set vs compiler runtime.", default = "MT"})
    end

    on_load("windows", "linux", "macosx",function (package)
        if package:config("external_abseil") then
            package:add("deps", "abseil")
        end
        if not package:config("builtin_protobuf") then
            package:add("deps", "protobuf-cpp")
        end
    end)

    on_install("windows", "linux", "macosx", function (package)
        local configs = {}
        table.insert(configs, "-DSPM_ENABLE_SHARED=" .. (package:config("shared") and "ON" or "OFF"))
        table.insert(configs, "-DSPM_USE_EXTERNAL_ABSL=" .. (package:config("external_abseil") and "ON" or "OFF"))
        table.insert(configs, "-DSPM_USE_BUILTIN_PROTOBUF=" .. (package:config("builtin_protobuf") and "ON" or "OFF"))
        import("package.tools.cmake").install(package, configs)
    end)

    on_test(function (package)
        assert(package:check_cxxsnippets({test = [[
            #include <iostream>
            void test(int args, char** argv) {
                sentencepiece::SentencePieceProcessor processor;
                const auto status = processor.Load("//path/to/model.model");
                if (!status.ok()) {
                    std::cerr << status.ToString() << std::endl;
                }
            }
        ]]}, {configs = {languages = "c++17"}, includes = "sentencepiece_processor.h"}))
    end)

package("ctranslate2")
	add_urls(--"https://github.com/OpenNMT/CTranslate2/archive/refs/tags/$(version).tar.gz",
			 "https://github.com/OpenNMT/CTranslate2.git")
	add_versions("v4.7.1", "64beb499c1e33500a691dfbe10cc5def7b914413c7f7c2830b0e0d8d544a9f7e") -- SHA256
	add_deps("cmake", {configs = {shared = true}})
	on_install(function (package)
        local configs = {}
        table.insert(configs, "-DCMAKE_BUILD_TYPE=" .. (package:debug() and "Debug" or "Release"))
        table.insert(configs, "-DBUILD_SHARED_LIBS=" .. (package:config("shared") and "ON" or "OFF"))
        table.insert(configs, "-DWITH_CUDA=ON")
        --table.insert(configs, "-DWITH_CUDNN=ON")
        --table.insert(configs, "-DWITH_ACCELERATE=ON")
        table.insert(configs, "-DCUDA_DYNAMIC_LOADING=ON")
        table.insert(configs, "-DWITH_HIP=ON")
        table.insert(configs, "-DWITH_DNNL=ON")
        import("package.tools.cmake").install(package, configs)
    end)

target("UnityTranslateLib")
    set_kind("shared")
    add_files("src/*.cpp")
    add_packages("sentencepiece", "ctranslate2", "boost", "re2", "jnipp", "openmp")

--
-- If you want to known more usage about xmake, please see https://xmake.io
--
-- ## FAQ
--
-- You can enter the project directory firstly before building project.
--
--   $ cd projectdir
--
-- 1. How to build project?
--
--   $ xmake
--
-- 2. How to configure project?
--
--   $ xmake f -p [macosx|linux|iphoneos ..] -a [x86_64|i386|arm64 ..] -m [debug|release]
--
-- 3. Where is the build output directory?
--
--   The default output directory is `./build` and you can configure the output directory.
--
--   $ xmake f -o outputdir
--   $ xmake
--
-- 4. How to run and debug target after building project?
--
--   $ xmake run [targetname]
--   $ xmake run -d [targetname]
--
-- 5. How to install target to the system directory or other output directory?
--
--   $ xmake install
--   $ xmake install -o installdir
--
-- 6. Add some frequently-used compilation flags in xmake.lua
--
-- @code
--    -- add debug and release modes
--    add_rules("mode.debug", "mode.release")
--
--    -- add macro definition
--    add_defines("NDEBUG", "_GNU_SOURCE=1")
--
--    -- set warning all as error
--    set_warnings("all", "error")
--
--    -- set language: c99, c++11
--    set_languages("c99", "c++11")
--
--    -- set optimization: none, faster, fastest, smallest
--    set_optimize("fastest")
--
--    -- add include search directories
--    add_includedirs("/usr/include", "/usr/local/include")
--
--    -- add link libraries and search directories
--    add_links("tbox")
--    add_linkdirs("/usr/local/lib", "/usr/lib")
--
--    -- add system link libraries
--    add_syslinks("z", "pthread")
--
--    -- add compilation and link flags
--    add_cxflags("-stdnolib", "-fno-strict-aliasing")
--    add_ldflags("-L/usr/local/lib", "-lpthread", {force = true})
--
-- @endcode
--

