#pragma once

#include <filesystem>
#include <fstream>
#include <vector>
#include <thread>
#include <regex>
#include <json/json.h>

#include "cmdline.h"

namespace Nvmd
{
  void noblock_system(const std::string &cmd)
  {
    std::thread thread{[cmd]()
                       {
                         std::system(cmd.c_str());
                       }};

    thread.detach();
  }

  std::vector<std::string> stringSplit(const std::string &str, char delim)
  {
    std::stringstream ss(str);
    std::string item;
    std::vector<std::string> elems;
    while (std::getline(ss, item, delim))
    {
      if (!item.empty())
      {
        elems.push_back(item);
      }
    }
    return elems;
  }

  std::string readFileContent(const std::filesystem::path &path)
  {
    // Sanity check
    if (!std::filesystem::is_regular_file(path))
      return {};

    // Open the file
    // Note that we have to use binary mode as we want to return a string
    // representing matching the bytes of the file on the file system.
    std::ifstream file(path, std::ios::in | std::ios::binary);
    if (!file.is_open())
      return {};

    // Read contents
    std::string content{std::istreambuf_iterator<char>(file), std::istreambuf_iterator<char>()};

    // Close the file
    file.close();

    return content;
  }

  std::string getVersion(const std::string &nvmd)
  {
    const auto nvmdrc = std::filesystem::current_path() / ".nvmdrc";

    auto projectVersion = readFileContent(nvmdrc);

    if (projectVersion.empty())
    {
      // find global version
      const auto defaultFile = std::filesystem::path(nvmd) / "default";
      const auto version = readFileContent(defaultFile);

      return version;
    }

    return projectVersion;
  }

  std::string getNpmRootPerfix(const std::string &path, const std::string &tempFile)
  {
    const auto command = path + "node " + path + "npm " + "root -g >" + tempFile;
    std::system(command.data());
    auto content = readFileContent(tempFile);
    std::string::size_type pos = 0;
    pos = content.find("\n", pos);
    if (pos != std::string::npos)
    {
      content.erase(pos, 1);
    }
    return content;
  }

  std::vector<std::string> getPackages(int argc, char *argv[])
  {
    // create a parser
    cmdline::parser command;
    command.add("global", 'g', "global");
    command.parse_check(argc, argv);

    std::vector<std::string> packages = command.rest();
    // packages.erase(packages.begin());

    std::regex reg("@[0-9]|@latest");
    for (int i = 1; i < packages.size(); i++)
    {
      std::string pk = packages[i];
      std::smatch smatch;
      if (std::regex_search(pk, smatch, reg))
      {
        const auto index = smatch.position();
        packages[i] = pk.substr(0, index);
      }
    }

    return packages;
  }

  std::vector<std::string> getPackagesName(const std::string &perfix, const std::vector<std::string> &packages)
  {
    std::vector<std::string> names;
    for (const auto &package : packages)
    {
      const auto packageJson = perfix + "/" + package + "/package.json";
      if (std::filesystem::is_regular_file(packageJson))
      {
        std::ifstream file(packageJson, std::ifstream::binary);
        if (file.is_open())
        {
          Json::Reader reader;
          Json::Value json;

          if (reader.parse(file, json))
          {
            const std::string name = json["name"].asString();

            if (json["bin"].isString())
            {
              names.push_back(name);
            }
            else
            {
              auto keys = json["bin"].getMemberNames();
              for (const auto &key : keys)
              {
                names.push_back(key);
              }
            }
          }

          file.close();
        }
      }
    }

    return names;
  }

  bool existOnStrArray(const Json::Value &json, const std::string &target)
  {
    if (!json.isArray())
      return false;

    for (int i = 0; i < json.size(); i++)
    {
      const auto str = json[i].asString();
      if (str == target)
        return true;
    }

    return false;
  }

  void recordForInstallPackages(const std::string &version, const std::string &path, const std::vector<std::string> &packages)
  {
    if (!std::filesystem::is_regular_file(path))
    {
      // not exsit
      Json::Value json;
      for (const auto &package : packages)
      {
        json[package].append(version);
      }

      Json::FastWriter fw;
      const auto jsonStr = fw.write(json);

      std::ofstream ofile(path);
      ofile << jsonStr;
      ofile.close();

      return;
    }

    // exsit
    std::ifstream file(path, std::ifstream::binary);
    if (file.is_open())
    {
      Json::Reader reader;
      Json::Value json;

      if (reader.parse(file, json))
      {
        for (const auto &package : packages)
        {
          if (json.empty() || !json.isMember(package))
          {
            json[package].append(version);
          }
          else
          {
            const auto exist = existOnStrArray(json[package], version);
            if (!exist)
            {
              json[package].append(version);
            }
          }
        }
      }
      else
      {
        for (const auto &package : packages)
        {
          json[package].append(version);
        }
      }

      Json::FastWriter fw;
      const auto jsonStr = fw.write(json);

      std::ofstream ofile(path);
      ofile << jsonStr;

      file.close();
      ofile.close();
    }
  }

  bool recordForUninstallPackage(const std::string &version, const std::string &path, const std::string &package)
  {
    // not exsit
    if (!std::filesystem::is_regular_file(path))
      return true;

    // exsit
    std::ifstream file(path, std::ifstream::binary);

    if (!file.is_open())
      return true;

    Json::Reader reader;
    Json::Value json;

    if (!reader.parse(file, json))
      return true;

    if (json.empty() || !json.isMember(package))
      return true;

    auto versions = json[package];
    if (!versions.isArray() || versions.size() == 0)
      return true;

    int index = -1;
    for (int i = 0; i < versions.size(); i++)
    {
      const auto str = versions[i];
      if (str == version)
      {
        index = i;
      }
    }

    if (index == -1)
      return false;

    Json::Value removed;
    versions.removeIndex(index, &removed);
    json[package] = versions;
    const auto flag = versions.size() == 0 ? true : false;

    Json::FastWriter fw;
    const auto jsonStr = fw.write(json);

    std::ofstream ofile(path);
    ofile << jsonStr;

    file.close();
    ofile.close();

    return flag;
  }
}