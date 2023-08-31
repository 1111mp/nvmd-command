#include <iostream>
#include <stdlib.h>
#include <unistd.h>

#include "nvmd.h"

int main(int argc, char *argv[])
{
  std::string lib = argv[0];

  const std::string nvmd = getenv("NVMD_DIR");

  // example: Users/zhangyifan/.nvmd/bin/node --version
  if (lib.find(nvmd + "/bin") != std::string::npos)
  {
    std::vector<std::string> splits = Nvmd::stringSplit(lib, '/');
    lib = splits.back();
  }

  const auto version = Nvmd::getVersion(nvmd);
  if (version.empty())
  {
    std::cout << lib << ": command not found" << std::endl;
    return 0;
  }

  std::string path;

#if defined(NVMD_PLATFORM_WINDOWS)
  path = nvmd + "/versions/" + version + "/";
#else
  path = nvmd + "/versions/" + version + "/bin";
#endif

  std::string params;
  for (int i = 1; i < argc; i++)
  {
    params = params + " " + argv[i];
  }

  const auto installOrUninstall = (params.find("install") != std::string::npos) || (params.find("uninstall") != std::string::npos);
  const auto isGlobal = (params.find("-g") != std::string::npos) || (params.find("--global") != std::string::npos);

  // npm install -g or npm uninstall -g
  if (lib == "npm" && installOrUninstall && isGlobal)
  {
    path = path + "/";
    std::string command = path + lib + params;
    command = path + "node " + command;
    auto packages = Nvmd::getPackages(argc, argv);
    const auto commandName = packages[0];
    packages.erase(packages.begin());

    if (commandName == "install")
    {
      // npm install -g
      const auto code = std::system(command.data());
      if (code == 0)
      {

        // the dir of npm global installed
        const auto perfix = Nvmd::getNpmRootPerfix(path, nvmd + "/temp.txt");
        // get packages bin name
        const auto packagesName = Nvmd::getPackagesName(perfix, packages);

        Nvmd::recordForInstallPackages(version, nvmd + "/packages.json", packagesName);

        const std::string binDir = nvmd + "/bin";
        for (const auto &name : packagesName)
        {
          const auto alias = binDir + "/" + name;
          if (!std::filesystem::is_symlink(alias))
          {
            std::filesystem::create_symlink(binDir + "/nvmd", alias);
          }
        }
      }
    }

    if (commandName == "uninstall")
    {
      // npm uninstall -g
      // the dir of npm global installed
      const auto perfix = Nvmd::getNpmRootPerfix(path, nvmd + "/temp.txt");
      // get packages bin name
      const auto packagesName = Nvmd::getPackagesName(perfix, packages);

      const auto code = std::system(command.data());
      if (code == 0)
      {
        const std::string binDir = nvmd + "/bin";
        for (const auto &name : packagesName)
        {
          const auto alias = binDir + "/" + name;
          if (Nvmd::recordForUninstallPackage(version, nvmd + "/packages.json", name))
          {
            std::filesystem::remove(alias);
          }
        }
      }
    }

    return 0;
  }

  std::string target = path + "/" + lib;

  char *args[argc + 1];
  for (int i = 0; i < argc; i++)
  {
    args[i] = argv[i];
  }

  args[argc] = nullptr;

  std::string envPath = getenv("PATH");
  auto newEnvPath = "PATH=" + path + ":" + envPath;

  char *envp[] = {newEnvPath.data(), nullptr};

  if (execve(target.c_str(), args, envp))
  {
    std::cout << lib << ": command not found" << std::endl;
  }

  return 0;
}
