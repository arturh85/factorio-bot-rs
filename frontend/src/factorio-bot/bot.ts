import type {FactorioEntity, FactorioPlayer, Position, Rect, World,} from "@/factorio-bot/types"
import {Direction, Entities, InventoryType} from "@/factorio-bot/types"
import {FactorioApi} from "@/factorio-bot/restApi";
import {
    distance,
    findFreeRect,
    groupByPosition,
    movePositionInDirection,
    positionStr,
    sortEntitiesByDistanceTo,
} from "@/factorio-bot/util"
import {Store} from "vuex"
import {State} from "@/store"
import {Task} from "@/factorio-bot/task";


/*
Source: https://factoriocheatsheet.com/#material-processing
 */
// Smelting iron, copper, and stone each take a base 3.2 seconds to finish.
const MS_PER_SMELT = 3200;
// Smelting steel takes base 16 seconds.
// const MS_PER_SMELT_STEEL = 16000
// Stone and Steel Furnaces consume 0.0225 coal/second.
const MS_COAL_BURN_TIME = 44444;

export const MAX_ITEM_INVENTORY = 300

export class FactorioBot {
    $store: Store<State>;
    playerId: number

    constructor(
        store: Store<State>,
        playerId: number
    ) {
        this.$store = store
        this.playerId = playerId
    }

    player(): FactorioPlayer {
        return this.$store.getters.getPlayer(this.playerId)
    }

    world(): World {
        return this.$store.state.world
    }

    mainInventory(itemName: string): number {
        return this.player().mainInventory[itemName] || 0;
    }

    async _findNearest(relativeTo: Position, count: number, name: string | null, entityName: string | null): Promise<FactorioEntity> {
        const searchRadius = 500;
        let target = null;
        for (let radius = 100; radius <= searchRadius; radius += 100) {
            let entities = (
                await FactorioApi.findEntities(relativeTo, radius, name, entityName)
            ).filter((entity) => entity.amount === null || entity.amount >= count);
            if (entities.length > 0) {
                entities = entities.sort(
                    sortEntitiesByDistanceTo(this.player().position)
                );
                target = entities[0];
                break;
            }
        }
        if (!target) {
            throw new Error(`no ${name}*${count} found within ${searchRadius}`);
        }
        return target;
    }

    async findNearestType(relativeTo: Position, name: string, count: number): Promise<FactorioEntity | null> {
        return await this._findNearest(relativeTo, count, null, name)
    }

    async findNearest(relativeTo: Position, name: string, count: number): Promise<Position | null> {
        const entity = await this._findNearest(relativeTo, count, name, null)
        if (entity) {
            return entity.position
        } else {
            return null
        }
    }

    async findNearestRect(
        relativeTo: Position,
        name: string,
        width: number,
        height: number,
        excludePositions: Position[] = []
    ): Promise<Position | null> {
        const nearestPosition = await this.findNearest(relativeTo, name, 200)
        if (!nearestPosition) {
            return null
        }
        const entities = await FactorioApi.findEntities(
            nearestPosition,
            40
        );
        const freePosition = findFreeRect(entities, this.$store.state.entityPrototypes, name, nearestPosition, width, height, excludePositions);
        if (!freePosition) {
            throw new Error(
                `no free ${name} field ${width}x${height} found`
            );
        }
        return freePosition
    }

    async tryMineNearest(name: string, count: number): Promise<void> {
        this.$store.commit('playerWorkStarted', this.playerId)
        try {
            await this.mineNearest(name, count);
        } catch (err) {
        } finally {
            this.$store.commit('playerWorkFinished', this.playerId)
        }
    }

    async tryMineNearestFrom(names: string[], count: number): Promise<void> {
        this.$store.commit('playerWorkStarted', this.playerId)
        try {
            await this.mineNearestFrom(names, count);
        } catch (err) {
        } finally {
            this.$store.commit('playerWorkFinished', this.playerId)
        }
    }

    async mineNearestFrom(names: string[], count: number): Promise<void> {
        this.$store.commit('playerWorkStarted', this.playerId)
        try {
            const targets = await Promise.all(names.map(name => this.findNearest(this.player().position, name, count)))
            if (!targets) {
                throw new Error(`not found: ${names.join(', ')}`)
            }
            let nearestTarget = null;
            let nearestName = null;
            let nearestDistance = 9999;
            const playerPosition = this.player().position
            for (let i = 0; i < targets.length; i++) {
                const d = distance(targets[i] as Position, playerPosition);
                if (d < nearestDistance) {
                    nearestDistance = d
                    nearestTarget = targets[i]
                    nearestName = names[i]
                }
            }
            if (!nearestTarget || !nearestName) {
                throw new Error(`not found: ${names.join(', ')}`)
            }
            return await this.mine(nearestTarget, nearestName, count)
        } catch (err) {
        } finally {
            this.$store.commit('playerWorkFinished', this.playerId)
        }
    }

    async mineNearestType(name: string, count: number): Promise<void> {
        const target = await this.findNearestType(this.player().position, name, count);
        if (target) {
            return await this.mine(target.position, target.name, count)
        } else {
            throw new Error(`not found: ${name}`)
        }
    }

    async mineNearest(name: string, count: number): Promise<void> {
        const target = await this.findNearest(this.player().position, name, count);
        if (target) {
            return await this.mine(target, name, count)
        } else {
            throw new Error(`not found: ${name}`)
        }
    }

    async mine(target: Position, name: string, count: number): Promise<void> {
        this.$store.commit('playerWorkStarted', this.playerId)
        try {
            const player = await FactorioApi.mine(
                this.playerId,
                target,
                name,
                count
            );
            if (player && player.playerId == this.playerId) {
                this.$store.commit('updatePlayer', player);
            } else {
                throw new Error('invalid response')
            }
        } finally {
            this.$store.commit('playerWorkFinished', this.playerId)
        }
    }

    async move(target: Position, radius: number): Promise<void> {
        this.$store.commit('playerWorkStarted', this.playerId)
        try {
            const player = await FactorioApi.move(
                this.playerId,
                target,
                radius
            );
            if (player && player.playerId == this.playerId) {
                this.$store.commit('updatePlayer', player);
            } else {
                throw new Error('invalid response')
            }
        } finally {
            this.$store.commit('playerWorkFinished', this.playerId)
        }
    }

    log(str: string, ...args: unknown[]): void {
        console.log(`[player ${this.player().playerId}] ${str}`, ...args);
    }

    async craft(recipeName: string, count: number): Promise<void> {
        this.$store.commit('playerWorkStarted', this.playerId)
        try {
            const recipe = this.$store.state.recipes[recipeName]
            if (recipe && recipe.products[0].amount > 1) {
                count = Math.ceil(count / recipe.products[0].amount)
            }
            const player = await FactorioApi.craft(
                this.playerId,
                recipeName,
                count
            );
            if (player && player.playerId == this.playerId) {
                this.$store.commit('updatePlayer', player);
            } else {
                throw new Error('invalid response')
            }
        } finally {
            this.$store.commit('playerWorkFinished', this.playerId)
        }
    }

    async placeEntity(
        itemName: string,
        placePosition: Position,
        placeDirection: number
    ): Promise<FactorioEntity> {
        this.$store.commit('playerWorkStarted', this.playerId)
        try {
            const result = await FactorioApi.placeEntity(
                this.playerId,
                itemName,
                placePosition,
                placeDirection
            );
            if (result.player && result.player.playerId == this.playerId) {
                this.$store.commit('updatePlayer', result.player);
            } else {
                throw new Error('invalid response')
            }
            return result.entity;
        } finally {
            this.$store.commit('playerWorkFinished', this.playerId)
        }
    }

    async reviveGhost(
        ghostEntity: FactorioEntity
    ): Promise<FactorioEntity> {
        this.$store.commit('playerWorkStarted', this.playerId)
        try {
            if (!ghostEntity.ghostName) {
                throw new Error("cannot revive non ghosts")
            }
            const result = await FactorioApi.reviveGhost(
                this.playerId,
                ghostEntity.ghostName,
                ghostEntity.position
            );
            if (result.player && result.player.playerId == this.playerId) {
                this.$store.commit('updatePlayer', result.player);
            } else {
                throw new Error('invalid response')
            }
            return result.entity;
        } finally {
            this.$store.commit('playerWorkFinished', this.playerId)
        }
    }

    async transferItemsTo(
        targetPlayerId: number,
        itemName: string,
        itemCount: number
    ): Promise<void> {
        this.$store.commit('playerWorkStarted', this.playerId)
        try {
            const targetPlayer: FactorioPlayer = this.$store.getters.getPlayer(targetPlayerId)
            const characterEntity = await FactorioApi.findEntities(targetPlayer.position, 1, 'character')
            if (characterEntity.length > 0) {
                return await this.insertToInventory('character', characterEntity[0].position, InventoryType.chest_or_fuel, itemName, itemCount)
            }
        } finally {
            this.$store.commit('playerWorkFinished', this.playerId)
        }
    }

    async insertToInventory(
        entityName: string,
        entityPosition: Position,
        inventoryType: InventoryType,
        itemName: string,
        itemCount: number
    ): Promise<void> {
        this.$store.commit('playerWorkStarted', this.playerId)
        try {
            const player = await FactorioApi.insertToInventory(
                this.player().playerId,
                entityName,
                entityPosition,
                inventoryType,
                itemName,
                itemCount
            );
            if (player && player.playerId == this.playerId) {
                this.$store.commit('updatePlayer', player);
            }
        } finally {
            this.$store.commit('playerWorkFinished', this.playerId)
        }
    }

    async removeFromInventory(
        entityName: string,
        entityPosition: Position,
        inventoryType: InventoryType,
        itemName: string,
        itemCount: number
    ): Promise<void> {
        this.$store.commit('playerWorkStarted', this.playerId)
        try {
            const player = await FactorioApi.removeFromInventory(
                this.player().playerId,
                entityName,
                entityPosition,
                inventoryType,
                itemName,
                itemCount
            );
            if (player && player.playerId == this.playerId) {
                this.$store.commit('updatePlayer', player);
            }
        } finally {
            this.$store.commit('playerWorkFinished', this.playerId)
        }
    }

    async cheatItem(itemName: string, itemCount: number): Promise<void> {
        this.$store.commit('playerWorkStarted', this.playerId)
        try {
            itemCount -= this.mainInventory(itemName);
            if (itemCount > 0) {
                const player = await FactorioApi.cheatItem(
                    this.player().playerId,
                    itemName,
                    itemCount
                );
                if (player && player.playerId == this.playerId) {
                    this.$store.commit('updatePlayer', player);
                } else {
                    throw new Error('invalid response')
                }
            }
        } finally {
            this.$store.commit('playerWorkFinished', this.playerId)
        }
    }

    async placeBlueprint(
        blueprint: string,
        position: Position,
        direction: number,
        forceBuild = false,
        inventoryPlayerIds: number[]
    ): Promise<FactorioEntity[]> {
        this.$store.commit('playerWorkStarted', this.playerId)
        try {
            const result = await FactorioApi.placeBlueprint(
                this.playerId,
                blueprint,
                position,
                direction,
                forceBuild,
                false,
                inventoryPlayerIds
            );
            if (result.player && result.player.playerId == this.playerId) {
                this.$store.commit('updatePlayer', result.player);
            } else {
                throw new Error('invalid response')
            }
            return result.entities;
        } finally {
            this.$store.commit('playerWorkFinished', this.playerId)
        }
    }

    async cheatBlueprint(
        blueprint: string,
        position: Position,
        direction: number,
        forceBuild = false
    ): Promise<FactorioEntity[]> {
        this.$store.commit('playerWorkStarted', this.playerId)
        try {
            const result = await FactorioApi.cheatBlueprint(
                this.playerId,
                blueprint,
                position,
                direction,
                forceBuild
            );
            if (result.player && result.player.playerId == this.playerId) {
                this.$store.commit('updatePlayer', result.player);
            } else {
                throw new Error('invalid response')
            }
            return result.entities;
        } finally {
            this.$store.commit('playerWorkFinished', this.playerId)
        }
    }
}
