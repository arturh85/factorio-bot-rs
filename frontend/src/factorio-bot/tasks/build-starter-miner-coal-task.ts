import {FactorioBot} from "@/factorio-bot/bot";
import {Store} from "vuex";
import {State} from "@/store";
import {createTask, executeTask, Task, taskRunnerByType, TaskStatus, updateTaskStatus} from "@/factorio-bot/task";
import {entityRect, placeEntitiesForCoalMinerLoop, sortBotsByInventory} from "@/factorio-bot/util";
import {Entities, InventoryType, Position, Rect, StarterCoalLoop, StarterMinerFurnace} from "@/factorio-bot/types";
import {createCraftTask} from "@/factorio-bot/tasks/craft-task";
import {createGatherTask} from "@/factorio-bot/tasks/gather-task";

const TASK_TYPE = 'build-starter-miner-coal'
const minerName = Entities.burnerMiningDrill;
const fuelName = Entities.coal;

type TaskData = {
    loopCount: number,
}

async function executeThisTask(store: Store<State>, bots: FactorioBot[], task: Task): Promise<StarterCoalLoop[]> {
    const data: TaskData = task.data as TaskData
    if (bots.length === 0) {
        throw new Error("no bots?")
    }
    // sort by already has correct item
    // bots.sort(sortBotsByInventory([minerName, furnaceName]))
    const bot = bots[0]
    const botMiners = bot.mainInventory(minerName)
    const subtasks: Task[] = []
    if (botMiners < data.loopCount) {
        const subtask = await createCraftTask(store, minerName, data.loopCount, false)
        store.commit('addSubTask', {id: task.id, task: subtask})
        subtasks.push(subtask)
    }
    if (subtasks.length > 0) {
        store.commit('updateTask', updateTaskStatus(task, TaskStatus.WAITING));
        for (const subTask of subtasks) {
            await executeTask(store, bots, subTask)
        }
        store.commit('updateTask', updateTaskStatus(task, TaskStatus.STARTED));
    }
    const excludePositions = Object.keys(store.state.players).map(key => store.state.players[key].position)
    const coalFieldTopLeft = await bot.findNearestRect(
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
    const newCoalMiners = placeEntitiesForCoalMinerLoop(anchor, data.loopCount);
    const coalLoops = []
    for (const coalMiner of newCoalMiners) {
        const coalMinerEntity = await bot.placeEntity(
            minerName,
            coalMiner.position,
            coalMiner.direction
        );
        const coalLoop: StarterCoalLoop = {
            minerPosition: coalMinerEntity.position,
            minerType: minerName
        }
        store.commit("addStarterCoalLoop", coalLoop)
        coalLoops.push(coalLoop)
    }
    if (bot.mainInventory(Entities.coal) < 1) {
        const subtask = await createGatherTask(store, Entities.coal, 1)
        store.commit('addSubTask', {id: task.id, task: subtask})
        store.commit('updateTask', updateTaskStatus(task, TaskStatus.WAITING));
        await executeTask(store, bots, subtask)
        store.commit('updateTask', updateTaskStatus(task, TaskStatus.STARTED));
    }
    await bot.insertToInventory(
        minerName,
        newCoalMiners[0].position,
        InventoryType.chest_or_fuel,
        Entities.coal,
        1
    );
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