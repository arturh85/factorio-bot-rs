import type {FactorioEntity, FactorioPlayer, Position, Rect, World,} from "@/factorio-bot/types"
import {Direction, Entities, InventoryType} from "@/factorio-bot/types"
import {FactorioApi} from "@/factorio-bot/restApi";
import {
    distance,
    findFreeRect, groupByPosition,
    movePositionInDirection,
    positionEqual, positionStr,
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

export class FactorioBot {
    $store: Store<State>;
    playerId: number
    busyWith: Task | null = null;

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

    async placeOffshorePump(): Promise<FactorioEntity> {
        for (let radius = 300; radius < 1000; radius += 200) {
            let tiles = await FactorioApi.findTiles(
                this.player().position,
                radius,
                Entities.water
            );
            if (tiles.length === 0) {
                continue;
            }
            const groupedByPosition = groupByPosition(tiles)
            tiles = tiles.filter(tile => {
                // tile MUST NOT have water directly above
                const above = groupedByPosition[positionStr(movePositionInDirection(tile.position, Direction.north))]
                if (above) {
                    return false
                }
                // tile MUST have water directly left and right
                const left = groupedByPosition[positionStr(movePositionInDirection(tile.position, Direction.east))]
                const right = groupedByPosition[positionStr(movePositionInDirection(tile.position, Direction.west))]
                return left && right;
            })
            if (tiles.length === 0) {
                continue
            }
            tiles.sort(sortEntitiesByDistanceTo(this.player().position));
            for (const tile of tiles) {
                const position = movePositionInDirection(
                    tile.position,
                    Direction.north
                );
                const conflictArea: Rect = {
                    leftTop: {x: position.x , y: position.y - 10},
                    rightBottom: {x: position.x + 18 , y: position.y},
                }
                const conflictEntities = await FactorioApi.findEntitiesInArea(conflictArea);
                if (conflictEntities.length > 0) {
                    // console.log('found conflicts for', tile.position, conflictArea, conflictEntities);
                    continue;
                }
                const conflictTiles = await FactorioApi.findTilesInArea(conflictArea);
                if (conflictTiles.filter(tile => tile.playerCollidable).length > 0) {
                    // console.log('found conflicts for', tile.position, conflictArea, conflictTiles);
                    continue;
                }
                try {
                    return await this.placeEntity(
                        Entities.offshorePump,
                        position,
                        Direction.south
                    );
                } catch (err) {
                }
            }
            throw Error(`failed to place ${Entities.offshorePump}`);
        }
        throw new Error("no nearby water found");
    }

    mainInventory(itemName: string): number {
        return this.player().mainInventory[itemName] || 0;
    }

    async _findNearest(count: number, name: string|null, entityName: string|null): Promise<FactorioEntity> {
        const searchRadius = 500;
        let target = null;
        for (let radius = 100; radius <= searchRadius; radius += 100) {
            let entities = (
                await FactorioApi.findEntities(this.player().position, radius, name, entityName)
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

    async findNearestType(name: string, count: number): Promise<FactorioEntity|null> {
       return await this._findNearest(count, null, name)
    }
    async findNearest(name: string, count: number): Promise<Position|null> {
        const entity = await this._findNearest(count, name, null)
        if (entity) {
            return entity.position
        } else {
            return null
        }
    }

    async findNearestRect(
        name: string,
        width: number,
        height: number,
        excludePositions: Position[] = []
    ): Promise<Position | null> {
        const nearestPosition = await this.findNearest(name, 200)
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
        try {
            await this.mineNearest(name, count);
        } catch (err) {
        }
    }

    async tryMineNearestFrom(names: string[], count: number): Promise<void> {
        try {
            await this.mineNearestFrom(names, count);
        } catch (err) {
        }
    }

    async mineNearestFrom(names: string[], count: number): Promise<void> {
        const targets = await Promise.all(names.map(name => this.findNearest(name, count)))
        if (!targets) {
            throw new Error(`not found: ${names.join(', ')}`)
        }
        let nearestTarget = null;
        let nearestName = null;
        let nearestDistance = 9999;
        const playerPosition = this.player().position
        for(let i=0; i<targets.length; i++) {
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
    }

    async mineNearestType(name: string, count: number): Promise<void> {
        const target = await this.findNearestType(name, count);
        if (target) {
            return await this.mine(target.position, target.name, count)
        } else {
            throw new Error(`not found: ${name}`)
        }
    }
    async mineNearest(name: string, count: number): Promise<void> {
        const target = await this.findNearest(name, count);
        if (target) {
            return await this.mine(target, name, count)
        } else {
            throw new Error(`not found: ${name}`)
        }
    }

    async mine(target: Position, name: string, count: number): Promise<void> {
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
    }

    async move(target: Position, radius: number): Promise<void> {
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
    }

    log(str: string, ...args: unknown[]): void {
        console.log(`[player ${this.player().playerId}] ${str}`, ...args);
    }

    async craft(recipeName: string, count: number): Promise<void> {
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
    }

    async placeEntity(
        itemName: string,
        _placePosition: Position,
        placeDirection: number
    ): Promise<FactorioEntity> {
        const result = await FactorioApi.placeEntity(
            this.playerId,
            itemName,
            _placePosition,
            placeDirection
        );
        if (result.player && result.player.playerId == this.playerId) {
            this.$store.commit('updatePlayer', result.player);
        } else {
            throw new Error('invalid response')
        }
        return result.entity;
    }

    async transferItemsTo(
        targetPlayerId: number,
        itemName: string,
        itemCount: number
    ): Promise<void> {
        const targetPlayer: FactorioPlayer = this.$store.getters.getPlayer(targetPlayerId)
        const characterEntity = await FactorioApi.findEntities(targetPlayer.position, 1, 'character')
        if (characterEntity.length > 0) {
            return await this.insertToInventory('character', characterEntity[0].position, InventoryType.chest_or_fuel, itemName, itemCount)
        }
    }

    async insertToInventory(
        entityName: string,
        entityPosition: Position,
        inventoryType: InventoryType,
        itemName: string,
        itemCount: number
    ): Promise<void> {
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
    }

    async removeFromInventory(
        entityName: string,
        entityPosition: Position,
        inventoryType: InventoryType,
        itemName: string,
        itemCount: number
    ): Promise<void> {
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
    }

    async cheatItem(itemName: string, itemCount: number): Promise<void> {
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
    }

    async placeBlueprint(
        blueprint: string,
        position: Position,
        direction: number,
        forceBuild = false
    ): Promise<FactorioEntity[]> {
        const result = await FactorioApi.placeBlueprint(
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
    }

    async cheatBlueprint(
        blueprint: string,
        position: Position,
        direction: number,
        forceBuild = false
    ): Promise<FactorioEntity[]> {
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
    }
}
