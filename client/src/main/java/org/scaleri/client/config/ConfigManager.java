package org.scaleri.client.config;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import java.io.IOException;
import java.io.Reader;
import java.io.Writer;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import net.fabricmc.loader.api.FabricLoader;
import org.scaleri.client.ScaleriClient;
import org.scaleri.client.module.HudModule;
import org.scaleri.client.module.Module;
import org.scaleri.client.module.ModuleManager;
import org.scaleri.client.setting.Setting;

/** Saves and loads every module's state to {@code config/scaleri-client.json}. */
public class ConfigManager {
    private static final Gson GSON = new GsonBuilder().setPrettyPrinting().create();

    private final Path file = FabricLoader.getInstance().getConfigDir().resolve("scaleri-client.json");
    private final ModuleManager modules;

    public ConfigManager(ModuleManager modules) {
        this.modules = modules;
    }

    public void load() {
        if (!Files.exists(file)) {
            return;
        }
        try (Reader reader = Files.newBufferedReader(file, StandardCharsets.UTF_8)) {
            JsonElement root = JsonParser.parseReader(reader);
            if (!root.isJsonObject() || !root.getAsJsonObject().has("modules")) {
                return;
            }
            JsonObject saved = root.getAsJsonObject().getAsJsonObject("modules");
            for (Module module : modules.all()) {
                if (saved.has(module.getId()) && saved.get(module.getId()).isJsonObject()) {
                    readModule(module, saved.getAsJsonObject(module.getId()));
                }
            }
        } catch (Exception e) {
            ScaleriClient.LOGGER.error("Could not read {}, using defaults", file, e);
        }
    }

    private static void readModule(Module module, JsonObject json) {
        if (json.has("settings") && json.get("settings").isJsonObject()) {
            JsonObject settings = json.getAsJsonObject("settings");
            for (Setting<?> setting : module.getSettings()) {
                if (settings.has(setting.getId())) {
                    setting.fromJson(settings.get(setting.getId()));
                }
            }
        }
        if (module instanceof HudModule hud && json.has("x") && json.has("y")) {
            hud.setPosition(json.get("x").getAsDouble(), json.get("y").getAsDouble());
        }
        if (module.isToggleable() && json.has("enabled")) {
            module.setEnabled(json.get("enabled").getAsBoolean());
        }
    }

    public void save() {
        JsonObject saved = new JsonObject();
        for (Module module : modules.all()) {
            JsonObject json = new JsonObject();
            json.addProperty("enabled", module.isEnabled());
            if (module instanceof HudModule hud) {
                json.addProperty("x", hud.getX());
                json.addProperty("y", hud.getY());
            }
            JsonObject settings = new JsonObject();
            for (Setting<?> setting : module.getSettings()) {
                settings.add(setting.getId(), setting.toJson());
            }
            json.add("settings", settings);
            saved.add(module.getId(), json);
        }
        JsonObject root = new JsonObject();
        root.addProperty("version", 1);
        root.add("modules", saved);

        try {
            Files.createDirectories(file.getParent());
            Path tmp = file.resolveSibling(file.getFileName() + ".tmp");
            try (Writer writer = Files.newBufferedWriter(tmp, StandardCharsets.UTF_8)) {
                GSON.toJson(root, writer);
            }
            Files.move(tmp, file, StandardCopyOption.REPLACE_EXISTING, StandardCopyOption.ATOMIC_MOVE);
        } catch (IOException e) {
            ScaleriClient.LOGGER.error("Could not save {}", file, e);
        }
    }
}
