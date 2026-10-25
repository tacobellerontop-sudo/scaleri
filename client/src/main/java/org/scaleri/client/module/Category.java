package org.scaleri.client.module;

public enum Category {
    HUD("HUD"),
    PVP("PvP"),
    VISUAL("Visual"),
    MISC("Misc");

    private final String displayName;

    Category(String displayName) {
        this.displayName = displayName;
    }

    public String getDisplayName() {
        return displayName;
    }
}
