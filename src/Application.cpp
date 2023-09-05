#include <iostream>
#include <stdlib.h>
#include <unistd.h>
#include <process.h>

#include "nvmd.h"

int main(int argc, char *argv[])
{
  std::string lib = argv[0];

  const std::string nvmd = getenv("NVMD_DIR");

#if defined(NVMD_PLATFORM_WINDOWS)
  // example: Users/zhangyifan/.nvmd/bin/node --version
  if (lib.find("\\") != std::string::npos)
  {
    std::vector<std::string> splits = Nvmd::stringSplit(lib, '\\');
    lib = splits.back();
  }

  if (lib.find(".exe") != std::string::npos)
  {
    std::regex reg(".exe");
    std::smatch smatch;
    if (std::regex_search(lib, smatch, reg))
    {
      const auto index = smatch.position();
      lib = lib.substr(0, index);
    }
  }
#else
  // example: Users/zhangyifan/.nvmd/bin/node --version
  if (lib.find(nvmd + Nvmd::block + "bin") != std::string::npos)
  {
    std::vector<std::string> splits = Nvmd::stringSplit(lib, Nvmd::block);
    lib = splits.back();
  }
#endif

  const std::string version = Nvmd::getVersion(nvmd);
  if (version.empty())
  {
    std::cout << lib << ": command not found" << std::endl;
    return 0;
  }

#if defined(NVMD_PLATFORM_WINDOWS)
  std::string path = nvmd + "\\versions\\" + version;
#else
  std::string path = nvmd + "/versions/" + version + "/bin";
#endif

  std::string params;
  for (int i = 1; i < argc; i++)
  {
    params = params + " " + argv[i];
  }

#if defined(NVMD_PLATFORM_WINDOWS)
  std::string target = path + Nvmd::block + lib;

  const char *args[argc + 1];
  args[0] = target.c_str();
  for (int i = 1; i < argc; i++)
  {
    args[i] = argv[i];
  }

  args[argc] = nullptr;

  std::string envPath = getenv("PATH");
  std::string newEnvPath = "PATH=" + path + ";" + envPath;
  const char *envp[] = {newEnvPath.c_str(), nullptr};

  const auto installOrUninstall = (params.find("install") != std::string::npos) || (params.find("uninstall") != std::string::npos);
  const auto isGlobal = (params.find("-g") != std::string::npos) || (params.find("--global") != std::string::npos);

  // npm install -g or npm uninstall -g
  if (lib == "npm" && installOrUninstall && isGlobal)
  {
    path = path + Nvmd::block;
    // std::string command = path + lib;
    // command = command;
    auto packages = Nvmd::getPackages(argc, argv);
    const auto commandName = packages[0];
    packages.erase(packages.begin());

    if (commandName == "install")
    {
      // npm install -g
      const auto code = _spawnve(_P_WAIT, target.c_str(), args, envp);
      if (code == 0)
      {
        // the dir of npm global installed
        const auto perfix = Nvmd::getNpmRootPerfix(path, nvmd + Nvmd::block + "temp.txt");
        // get packages bin name
        const auto packagesName = Nvmd::getPackagesName(perfix, packages);

        Nvmd::recordForInstallPackages(version, nvmd + Nvmd::block + "packages.json", packagesName);

        const std::string binDir = nvmd + Nvmd::block + "bin";
        for (const auto &name : packagesName)
        {
          const auto alias = binDir + Nvmd::block + name + ".exe";
          if (!std::filesystem::is_regular_file(alias))
          {
            std::filesystem::copy_file(binDir + Nvmd::block + "nvmd.exe", alias);
          }
        }
      }
    }

    if (commandName == "uninstall")
    {
      // npm uninstall -g
      // the dir of npm global installed
      const auto perfix = Nvmd::getNpmRootPerfix(path, nvmd + Nvmd::block + "temp.txt");
      // get packages bin name
      const auto packagesName = Nvmd::getPackagesName(perfix, packages);

      const auto code = _spawnve(_P_WAIT, target.c_str(), args, envp);
      if (code == 0)
      {
        const std::string binDir = nvmd + Nvmd::block + "bin";
        for (const auto &name : packagesName)
        {
          const auto alias = binDir + Nvmd::block + name + ".exe";
          if (Nvmd::recordForUninstallPackage(version, nvmd + Nvmd::block + "packages.json", name))
          {
            std::filesystem::remove(alias);
          }
        }
      }
    }

    return 0;
  }

  auto hProcess = _spawnve(_P_NOWAIT, target.c_str(), args, envp);

  int termstat;
  _cwait(&termstat, hProcess, _WAIT_GRANDCHILD);

  if (termstat)
  {
    std::cout << lib << ": command not found" << std::endl;
  }
#else
  const auto installOrUninstall = (params.find("install") != std::string::npos) || (params.find("uninstall") != std::string::npos);
  const auto isGlobal = (params.find("-g") != std::string::npos) || (params.find("--global") != std::string::npos);

  // npm install -g or npm uninstall -g
  if (lib == "npm" && installOrUninstall && isGlobal)
  {
    path = path + Nvmd::block;
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
        const auto perfix = Nvmd::getNpmRootPerfix(path, nvmd + Nvmd::block + "temp.txt");
        // get packages bin name
        const auto packagesName = Nvmd::getPackagesName(perfix, packages);

        Nvmd::recordForInstallPackages(version, nvmd + Nvmd::block + "packages.json", packagesName);

        const std::string binDir = nvmd + Nvmd::block + "bin";
        for (const auto &name : packagesName)
        {
          const auto alias = binDir + Nvmd::block + name;
          if (!std::filesystem::is_symlink(alias))
          {
            std::filesystem::create_symlink(binDir + Nvmd::block + "nvmd", alias);
          }
        }
      }
    }

    if (commandName == "uninstall")
    {
      // npm uninstall -g
      // the dir of npm global installed
      const auto perfix = Nvmd::getNpmRootPerfix(path, nvmd + Nvmd::block + "temp.txt");
      // get packages bin name
      const auto packagesName = Nvmd::getPackagesName(perfix, packages);

      const auto code = std::system(command.data());
      if (code == 0)
      {
        const std::string binDir = nvmd + Nvmd::block + "bin";
        for (const auto &name : packagesName)
        {
          const auto alias = binDir + Nvmd::block + name;
          if (Nvmd::recordForUninstallPackage(version, nvmd + Nvmd::block + "packages.json", name))
          {
            std::filesystem::remove(alias);
          }
        }
      }
    }

    return 0;
  }

  std::string target = path + Nvmd::block + lib;

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
#endif

  return 0;
}
