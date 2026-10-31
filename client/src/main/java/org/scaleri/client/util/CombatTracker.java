package org.scaleri.client.util;

import net.minecraft.client.MinecraftClient;
import net.minecraft.entity.Entity;
import net.minecraft.entity.LivingEntity;
import net.minecraft.util.hit.EntityHitResult;
import net.minecraft.util.hit.HitResult;

/** Tracks your last hit's reach and your current combo for the PvP HUD modules. */
public final class CombatTracker {
    private static final long RESET_MS = 2500L;

    private static double lastReach = -1;
    private static long lastReachTime;
    private static int combo;
    private static long lastComboTime;

    private static LivingEntity pendingTarget;
    private static int pendingTicks;
    private static int lastOwnHurtTime;

    private CombatTracker() {
    }

    /** Called when the local player swings at an entity. */
    public static void onAttack(Entity target) {
        MinecraftClient mc = MinecraftClient.getInstance();
        if (mc.player == null) {
            return;
        }
        HitResult hit = mc.crosshairTarget;
        if (hit instanceof EntityHitResult entityHit && entityHit.getEntity() == target) {
            lastReach = entityHit.getPos().distanceTo(mc.player.getEyePos());
            lastReachTime = System.currentTimeMillis();
        }
        if (target instanceof LivingEntity living) {
            // Only count the hit once the server confirms it by making the target flash red.
            pendingTarget = living;
            pendingTicks = 4;
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
            if (pendingTarget.hurtTime > 0 && pendingTarget.hurtTime >= pendingTarget.maxHurtTime - 3) {
                combo++;
                lastComboTime = System.currentTimeMillis();
                pendingTarget = null;
            } else if (--pendingTicks <= 0) {
                pendingTarget = null;
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

    /** Distance of the last hit in blocks, or -1 when it is too old to show. */
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
