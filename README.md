# package-kpz

A (hopefully) faster alternative to the sample gulpfile used in https://github.com/bywatersolutions/dev-koha-plugin-kitchen-sink.

## Usage

The tool requires the following parameters:

- `--release-filename` (or `-r`): The filename for the output release file.
- `--pm-file-path` (or `-p`): The full path to the PM file containing the plugin's main module.
- `--translations-dir` (or `-t`, optional): The directory containing the .po translation files to be converted to JSON.

### Example

```sh
$ ./package-kpz -r "Koha-Plugin-Example" -p "Koha/Plugin/Com/LMSCloud/Example.pm" -t "translations"
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
3. __convert_translations__: Converts the .po translation files to JSON and saves them in the dist directory.
4. __substitute_strings__: Updates the plugin version and release date in the PM file using the values from package.json.
5. __create_zip__: Creates a ZIP file containing the plugin's contents and saves it with the specified release filename.
6. __cleanup__: Removes the temporary dist directory created during the process.
