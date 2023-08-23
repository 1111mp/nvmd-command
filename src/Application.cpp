#include <iostream>

#include "nvmd.h"

int main(int argc, char *argv[])
{
  const std::string lib = argv[0];
  std::string params;
  for (int i = 1; i < argc; i++)
  {
    params = params + " " + argv[i];
  }

  const std::string nvmd = getenv("NVMD_DIR");
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
  path = nvmd + "/versions/" + version + "/bin/";
#endif

  // generate command
  std::string command = path + lib + " " + params;
  if (lib != "node")
  {
    command = path + "node " + command;
  }

  if (lib != "npm")
  {
    std::system(command.data());
    return 0;
  }

  const auto isGlobal = (params.find("-g") != std::string::npos) || (params.find("--global") != std::string::npos);

  if (!isGlobal)
  {
    std::system(command.data());
    return 0;
  }

  // npm install -g or npm uninstall -g
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
