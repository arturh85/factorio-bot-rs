import {FactorioBot} from "@/factorio-bot/bot";
import {Store} from "vuex";
import {State} from "@/store";
import {createTask, registerTaskRunner, Task} from "@/factorio-bot/task";
import {
    groupByPosition,
    movePositionInDirection,
    positionStr,
    sortBotsByInventory,
    sortEntitiesByDistanceTo
} from "@/factorio-bot/util";
import {Direction, Entities, FactorioEntity, Rect} from "@/factorio-bot/types";
import {buildBotQueueToCraft, processBotQueue} from "@/factorio-bot/bot-queue";
import {FactorioApi} from "@/factorio-bot/restApi";

const TASK_TYPE = 'build-starter-offshore-pump'

type TaskData = any

async function executeThisTask(store: Store<State>, bots: FactorioBot[], task: Task): Promise<FactorioEntity> {
    if (bots.length === 0) {
        throw new Error("no bots?")
    }
    bots.sort(sortBotsByInventory([Entities.offshorePump]))
    const firstBot = bots[0]

    const queue = await buildBotQueueToCraft(store, task, [firstBot], {
        [Entities.offshorePump]: 1,
        [Entities.pipe]: 1
    })
    await processBotQueue(store, queue, [firstBot])

    let offshorePump = null

    for (let radius = 300; radius < 1000; radius += 200) {
        let tiles = await FactorioApi.findTiles(
            {x: 0, y: 0},
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
        tiles.sort(sortEntitiesByDistanceTo(firstBot.player().position));
        for (const tile of tiles) {
            const position = movePositionInDirection(
                tile.position,
                Direction.north
            );
            const conflictArea: Rect = {
                leftTop: {x: position.x, y: position.y - 23},
                rightBottom: {x: position.x + 18, y: position.y},
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
                offshorePump = await firstBot.placeEntity(
                    Entities.offshorePump,
                    position,
                    Direction.south
                );
                break
            } catch (err) {
            }
        }
        if (!offshorePump) {
            throw Error(`failed to place ${Entities.offshorePump}`);
        } else {
            break
        }
    }
    if (!offshorePump) {
        throw new Error("no nearby water found");
    }
    store.commit("setStarterOffshorePump", offshorePump.position)
    await firstBot.placeEntity(
        Entities.pipe,
        {
            x: offshorePump.position.x,
            y: offshorePump.position.y - 1,
        },
        Direction.south
    );
    return offshorePump
}

registerTaskRunner(TASK_TYPE, executeThisTask)

export async function createBuildStarterOffshorePumpTask(store: Store<State>): Promise<Task> {
    const data: TaskData = {}
    return createTask(TASK_TYPE, `Build Starter Offshore Pump`, data)
}