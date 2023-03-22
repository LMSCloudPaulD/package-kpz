# package-kpz

A (hopefully) faster alternative to the sample gulpfile used in https://github.com/bywatersolutions/dev-koha-plugin-kitchen-sink.

## Usage

The tool requires the following parameters:

- `--release-filename` (or `-r`): The filename for the output release file.
- `--pm-file-path` (or `-p`): The full path to the PM file containing the plugin's main module.

### Example

```sh
$ ./package-kpz -r "Koha-Plugin-Example-v1.0.0.kpz" -p "dist/Koha/Plugin/Com/LMSCloud/EventManagement.pm"
```

This will create a release file named **Koha-Plugin-Example-v1.0.0.kpz** containing the contents of the plugin with the main module located in **dist/Koha/Plugin/Com/LMSCloud/EventManagement.pm**.

## Assumptions

When running this module, it is assumed that:

1. The working directory contains a **package.json** file with the plugin's version information.
2. The **Koha** directory, containing the plugin files to be packaged, is present in the working directory.
3. The specified PM file exists at the given path and has placeholders for the version (_{VERSION}_) and release date (_1900-01-01_).

## Documentation

The tool consists of several functions that perform the following tasks:

1. __build_directory__: Creates the dist directory.
2. __copy_files__: Copies the plugin files from the Koha directory to the dist directory.
3. __substitute_strings__: Updates the plugin version and release date in the PM file using the values from package.json.
4. __create_zip__: Creates a ZIP file containing the plugin's contents and saves it with the specified release filename.
5. __cleanup__: Removes the temporary dist directory created during the process.
