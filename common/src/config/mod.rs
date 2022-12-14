use std::io::Read;
use std::path::{Path, PathBuf};

pub struct ConfigurationManager<T>
where
    T: Sized,
{
    configuration: T,
    configuration_loader: ConfigurationLoader,
}

impl<T> ConfigurationManager<T>
where
    T: Default + serde::Serialize + serde::de::DeserializeOwned,
{
    pub fn load(
        application_folder: &Path,
        config_file_name: &str,
    ) -> anyhow::Result<ConfigurationManager<T>> {
        let configuration_loader = ConfigurationLoader::new(application_folder, config_file_name);

        let configuration = Self::load_configuration(&configuration_loader)?;
        Ok(ConfigurationManager {
            configuration,
            configuration_loader,
        })
    }

    fn load_configuration(configuration_loader: &ConfigurationLoader) -> anyhow::Result<T> {
        let configuration;
        if configuration_loader.config_exists() {
            configuration = configuration_loader.load_config()?;
        } else {
            configuration = T::default();
            configuration_loader.store_config(&configuration)?;
        }
        Ok(configuration)
    }

    pub fn persist_configuration(&self) -> anyhow::Result<()> {
        self.configuration_loader.store_config(&self.configuration)
    }

    pub fn get_configuration(&self) -> &T {
        &self.configuration
    }

    pub fn reload_configuration(&mut self) -> anyhow::Result<()> {
        let configuration = Self::load_configuration(&self.configuration_loader)?;
        self.configuration = configuration;
        Ok(())
    }

    pub fn set_configuration(&mut self, configuration: T) {
        self.configuration = configuration;
    }
}

struct ConfigurationLoader {
    config_file_path: PathBuf,
}

impl ConfigurationLoader {
    pub fn new(application_folder: &Path, config_file_name: &str) -> ConfigurationLoader {
        let mut config_file_path = application_folder.to_owned();
        config_file_path.push(config_file_name);
        ConfigurationLoader { config_file_path }
    }

    pub fn load_config<T>(&self) -> anyhow::Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let mut file = std::fs::File::open(&self.config_file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let config = serde_json::from_str(&contents)?;
        Ok(config)
    }

    pub fn config_exists(&self) -> bool {
        self.config_file_path.exists()
    }

    pub fn store_config<T>(&self, config: &T) -> anyhow::Result<()>
    where
        T: serde::Serialize,
    {
        let serialized_data = serde_json::to_string_pretty(config)?;
        std::fs::write(&self.config_file_path, serialized_data)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::fs;

    use super::*;

    const DEFAULT_STRING_VALUE: &str = "This is the default value";
    const CONFIG_FILE_NAME: &str = "config.json";

    #[derive(Serialize, Deserialize, Clone)]
    struct TestConfig {
        string_value: String,
    }

    impl Default for TestConfig {
        fn default() -> Self {
            TestConfig {
                string_value: DEFAULT_STRING_VALUE.to_owned(),
            }
        }
    }

    #[test]
    fn test_load_config_not_existing() {
        let path = fs::util::prepare_temp_folder().unwrap();
        let loader = ConfigurationLoader::new(&path, CONFIG_FILE_NAME);

        assert!(!loader.config_exists());

        let result = loader.load_config::<TestConfig>();
        assert!(result.is_err());
        fs::util::delete_temp_folder(&path).unwrap();
    }

    #[test]
    fn test_store_config() {
        let path = fs::util::prepare_temp_folder().unwrap();
        let loader = ConfigurationLoader::new(&path, CONFIG_FILE_NAME);

        const TEST_VALUE: &str = "Test value";
        let config = TestConfig {
            string_value: TEST_VALUE.to_owned(),
        };
        loader.store_config(&config).unwrap();

        let mut config_file_path = path.clone();
        config_file_path.push(CONFIG_FILE_NAME);

        assert!(config_file_path.exists());
        fs::util::delete_temp_folder(&path).unwrap();
    }

    #[test]
    fn test_store_and_load_config() {
        let path = fs::util::prepare_temp_folder().unwrap();
        let loader = ConfigurationLoader::new(&path, CONFIG_FILE_NAME);

        const TEST_VALUE: &str = "This is the expected text.";
        let config = TestConfig {
            string_value: TEST_VALUE.to_owned(),
        };
        loader.store_config(&config).unwrap();

        let loaded_config = loader.load_config::<TestConfig>().unwrap();

        assert_eq!(config.string_value, loaded_config.string_value);

        fs::util::delete_temp_folder(&path).unwrap();
    }

    #[test]
    fn test_create_config_manager() {
        let path = fs::util::prepare_temp_folder().unwrap();
        let manager = ConfigurationManager::<TestConfig>::load(&path, CONFIG_FILE_NAME).unwrap();

        assert_eq!(DEFAULT_STRING_VALUE, manager.configuration.string_value);

        fs::util::delete_temp_folder(&path).unwrap();
    }
}
