package org.scaleri.client.util;

import net.minecraft.client.MinecraftClient;
import net.minecraft.client.network.PlayerListEntry;
import net.minecraft.entity.Entity;
import net.minecraft.entity.LivingEntity;
import net.minecraft.util.hit.EntityHitResult;
import net.minecraft.util.hit.HitResult;
import net.minecraft.util.math.MathHelper;

/** Tracks your last hit's reach and your current combo for the PvP HUD modules. */
public final class CombatTracker {
    private static final long RESET_MS = 2500L;

    private static double lastReach = -1;
    private static long lastReachTime;
    private static int combo;
    private static long lastComboTime;

    private static LivingEntity pendingTarget;
    private static double pendingReach = -1;
    private static int pendingTicks;
    private static int pendingHurtTime;
    private static int lastOwnHurtTime;

    private CombatTracker() {
    }

    /** Called when the local player swings at an entity. */
    public static void onAttack(Entity target) {
        MinecraftClient mc = MinecraftClient.getInstance();
        if (mc.player == null) {
            return;
        }
        pendingReach = -1;
        HitResult hit = mc.crosshairTarget;
        if (hit instanceof EntityHitResult entityHit && entityHit.getEntity() == target) {
            pendingReach = entityHit.getPos().distanceTo(mc.player.getEyePos());
        }
        if (target instanceof LivingEntity living) {
            // Only count the hit once the server confirms it by making the target flash red.
            // The confirmation takes a round trip, so wait longer on laggy connections.
            pendingTarget = living;
            pendingHurtTime = living.hurtTime;
            pendingTicks = MathHelper.clamp(ping(mc) / 50 + 4, 4, 20);
        }
    }

    public static void tick() {
        MinecraftClient mc = MinecraftClient.getInstance();
        if (mc.player == null) {
            pendingTarget = null;
            combo = 0;
            return;
        }
        if (pendingTarget != null) {
            if (pendingTarget.hurtTime > pendingHurtTime) {
                combo++;
                lastComboTime = System.currentTimeMillis();
                if (pendingReach >= 0) {
                    lastReach = pendingReach;
                    lastReachTime = lastComboTime;
                }
                pendingTarget = null;
            } else if (--pendingTicks <= 0 || pendingTarget.isRemoved()) {
                pendingTarget = null;
            } else {
                pendingHurtTime = pendingTarget.hurtTime;
            }
        }
        int hurt = mc.player.hurtTime;
        if (hurt > lastOwnHurtTime) {
            combo = 0;
        }
        lastOwnHurtTime = hurt;
        if (combo > 0 && System.currentTimeMillis() - lastComboTime > RESET_MS) {
            combo = 0;
        }
    }

    private static int ping(MinecraftClient mc) {
        if (mc.getNetworkHandler() == null) {
            return 0;
        }
        PlayerListEntry entry = mc.getNetworkHandler().getPlayerListEntry(mc.player.getUuid());
        return entry == null ? 0 : entry.getLatency();
    }

    /** Distance of the last landed hit in blocks, or -1 when it is too old to show. */
    public static double getLastReach() {
        if (lastReach < 0 || System.currentTimeMillis() - lastReachTime > RESET_MS * 2) {
            return -1;
        }
        return lastReach;
    }

    public static int getCombo() {
        return combo;
    }
}
