package org.scaleri.client.module.misc;

import java.util.List;
import org.scaleri.client.module.Category;
import org.scaleri.client.module.Module;
import org.scaleri.client.setting.ModeSetting;
import org.scaleri.client.setting.NumberSetting;

/** Says gg for you when a game ends on popular minigame servers. */
public class AutoGgModule extends Module {
    /** End-of-game lines from common minigame servers (Hypixel, etc). */
    private static final List<String> TRIGGERS = List.of(
            "1st Killer - ",
            "1st Place - ",
            "Winner: ",
            " - Damage Dealt - ",
            "Winning Team -",
            "1st - ",
            "Winners: ",
            "Winner #1 (",
            "Top Survivors",
            "won the game!",
            "Reward Summary");

    private final ModeSetting message = add(new ModeSetting("Message", "gg", "gg", "GG", "gg wp", "Good game!", "gf"));
    private final ModeSetting channel = add(new ModeSetting("Send As", "Chat", "Chat", "/ac (Hypixel all chat)"));
    private final NumberSetting delay = add(new NumberSetting("Delay (seconds)", 1, 0, 5, 0.5));

    private int countdown = -1;
    private long lastSent;

    public AutoGgModule() {
        super("Auto GG", "Says gg for you when a minigame ends.", Category.MISC, false);
    }

    public void onGameMessage(String text) {
        if (System.currentTimeMillis() - lastSent < 10_000 || countdown >= 0) {
            return;
        }
        for (String trigger : TRIGGERS) {
            if (text.contains(trigger)) {
                countdown = (int) Math.round(delay.get() * 20);
                return;
            }
        }
    }

    @Override
    public void onTick() {
        if (countdown < 0) {
            return;
        }
        if (countdown-- > 0) {
            return;
        }
        if (mc.getNetworkHandler() != null) {
            if (channel.is("Chat")) {
                mc.getNetworkHandler().sendChatMessage(message.get());
            } else {
                mc.getNetworkHandler().sendChatCommand("ac " + message.get());
            }
            lastSent = System.currentTimeMillis();
        }
    }
}
