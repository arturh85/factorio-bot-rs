import {FactorioBot} from "@/factorio-bot/bot";
import {Store} from "vuex";
import {State} from "@/store";
import {createTask, registerTaskRunner, Task} from "@/factorio-bot/task";
import {movePositionInDirection, sortBotsByInventory, sortPositionsByDistanceTo} from "@/factorio-bot/util";
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
    const searchCenter = {x: 0, y: 0}
    await processBotQueue(store, queue, [firstBot])
    const placementOptions = await FactorioApi.findOffshorePumpPlacementOptions(searchCenter, Direction.north)
    placementOptions.sort(sortPositionsByDistanceTo(searchCenter));
    let offshorePump = null
    for (const placementOption of placementOptions) {
        const position = movePositionInDirection(
            placementOption,
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
            console.warn("failed to place offshore pump", err)
        }
    }
    if (!offshorePump) {
        throw Error(`failed to place ${Entities.offshorePump}`);
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

export async function createBuildStarterOffshorePumpTask(_store: Store<State>): Promise<Task> {
    const data: TaskData = {}
    return createTask(TASK_TYPE, `Build Starter Offshore Pump`, data)
}