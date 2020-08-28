import {FactorioBot} from "@/factorio-bot/bot";
import {Store} from "vuex";
import {State} from "@/store";
import {
    buildBotQueue, buildBotQueueToCraft,
    createTask,
    executeTask, processBotQueue,
    Task,
    taskRunnerByType,
    TaskStatus,
    updateTaskStatus
} from "@/factorio-bot/task";
import {entityRect, placeEntitiesForCoalMinerLoop, sortBotsByInventory} from "@/factorio-bot/util";
import {Entities, InventoryType, Position, Rect, StarterCoalLoop, StarterMinerFurnace} from "@/factorio-bot/types";
import {createCraftTask} from "@/factorio-bot/tasks/craft-task";
import {createGatherTask} from "@/factorio-bot/tasks/gather-task";
import {createPlaceStarterMinerFurnaceTask} from "@/factorio-bot/tasks/place-starter-miner-furnace";
import {createPlaceTask} from "@/factorio-bot/tasks/place-task";

const TASK_TYPE = 'build-starter-miner-coal'
const minerName = Entities.burnerMiningDrill;
const fuelName = Entities.coal;

type TaskData = {
    loopCount: number,
}

async function executeThisTask(store: Store<State>, bots: FactorioBot[], task: Task): Promise<StarterCoalLoop[]> {
    const data: TaskData = task.data as TaskData

    const firstBot = bots[0]
    const craftQueue = await buildBotQueueToCraft(store, task, bots, {
        minerName: data.loopCount
    })
    await processBotQueue(store, craftQueue, bots)

    const excludePositions = Object.keys(store.state.players).map(key => store.state.players[key].position)
    const coalFieldTopLeft = await firstBot.findNearestRect(
        Entities.coal,
        2 * 2,
        2 * 2,
        excludePositions
    );
    if (!coalFieldTopLeft) {
        store.commit('updateTask', updateTaskStatus(task, TaskStatus.FAILED));
        throw new Error(`failed to find ore field for ${Entities.coal}`)
    }
    const anchor = {...coalFieldTopLeft};
    anchor.x = Math.floor(anchor.x);
    anchor.y = Math.floor(anchor.y);

    const placeQueue = buildBotQueue(bots)
    const placePosition = {...anchor};
    let toPlace = data.loopCount

    const width = data.loopCount / 2
    for(const bot of bots) {
        const playerId = bot.playerId.toString()
        const botPlaceItems = Math.min(toPlace, Math.ceil(data.loopCount / bots.length))
        for (let i = 0; i < botPlaceItems; i++) {
            const i = data.loopCount - toPlace
            const y = i / width
            const x = i % width
            placePosition.y = anchor.y + y * 2;
            placePosition.x = anchor.x + x * 2;
            let direction = y === 0 ? 2 : 6; // right/east in first row, left/west in second
            // if right top corner
            if (y === 0 && x === width - 1) {
                direction = 4; // down/south
            } else if (y == 1 && x === 0) {
                direction = 0; // up/north
            }
            const subtask = await createPlaceTask(store, minerName, {...placePosition}, direction)
            store.commit('addSubTask', {id: task.id, task: subtask})
            placeQueue[playerId].push(subtask)
            toPlace -= 1
        }
    }

    const results = await processBotQueue(store, placeQueue, bots)
    const coalLoops = results.flatMap(result => {
        if (result && Array.isArray(result)) {
            return result
        } else {
            return []
        }
    }).map(entity => {
        const coalLoop: StarterCoalLoop = {
            minerPosition: entity.position,
            minerType: minerName
        }
        store.commit("addStarterCoalLoop", coalLoop)
        return coalLoop
    })
    // if (bot.mainInventory(Entities.coal) < 1) {
    //     const subtask = await createGatherTask(store, Entities.coal, 1)
    //     store.commit('addSubTask', {id: task.id, task: subtask})
    //     store.commit('updateTask', updateTaskStatus(task, TaskStatus.WAITING));
    //     await executeTask(store, bots, subtask)
    //     store.commit('updateTask', updateTaskStatus(task, TaskStatus.STARTED));
    // }
    // await bot.insertToInventory(
    //     minerName,
    //     newCoalMiners[0].position,
    //     InventoryType.chest_or_fuel,
    //     Entities.coal,
    //     1
    // );
    return coalLoops
}

taskRunnerByType[TASK_TYPE] = executeThisTask

export async function createBuildStarterMinerCoalTask(store: Store<State>, loopCount: number): Promise<Task> {
    if (loopCount % 2 !== 0) {
        throw new Error("only even number of starter coal miners supported")
    }
    const data: TaskData = {
        loopCount,
    }
    return createTask(TASK_TYPE, `Build Starter Miner Coal x ${loopCount}`, data)
}