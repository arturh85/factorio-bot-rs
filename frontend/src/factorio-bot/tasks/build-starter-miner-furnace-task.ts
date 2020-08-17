import {FactorioBot} from "@/factorio-bot/bot";
import {Store} from "vuex";
import {State} from "@/store";
import {createTask, executeTask, Task, taskRunnerByType, TaskStatus, updateTaskStatus} from "@/factorio-bot/task";
import {entityRect, sortBotsByInventory} from "@/factorio-bot/util";
import {Entities, Position, Rect, StarterMinerFurnace} from "@/factorio-bot/types";
import {createCraftTask} from "@/factorio-bot/tasks/craft-task";

const TASK_TYPE = 'build-starter-miner-furnace'
const minerName = Entities.burnerMiningDrill;
const furnaceName = Entities.stoneFurnace;

type TaskData = {
    oreName: string,
    plateName: string,
    minerSmelterCount: number,
}

async function executeThisTask(store: Store<State>, bots: FactorioBot[], task: Task): Promise<StarterMinerFurnace[]> {
    const data: TaskData = task.data as TaskData
    if (bots.length === 0) {
        throw new Error("no bots?")
    }
    // sort by already has correct item
    bots.sort(sortBotsByInventory([minerName, furnaceName]))
    const bot = bots[0]
    const subtasks: Task[] = []
    if (bot.mainInventory(minerName) < data.minerSmelterCount) {
        const subtask = await createCraftTask(store, minerName, data.minerSmelterCount)
        store.commit('addSubTask', {id: task.id, task: subtask})
        subtasks.push(subtask)
    }
    if (bot.mainInventory(furnaceName) < data.minerSmelterCount) {
        const subtask = await createCraftTask(store, furnaceName, data.minerSmelterCount)
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
    const oreFieldTopLeft = await bot.findNearestRect(
        data.oreName,
        2 * data.minerSmelterCount,
        4,
        excludePositions,
    );
    if (!oreFieldTopLeft) {
        store.commit('updateTask', updateTaskStatus(task, TaskStatus.FAILED));
        throw new Error(`failed to find ore field for ${data.oreName}`)
    }
    const anchor = {...oreFieldTopLeft};
    anchor.x = Math.floor(anchor.x);
    anchor.y = Math.floor(anchor.y);
    const minerFurnaces = []
    for (let x = 0; x < data.minerSmelterCount; x++) {
        const minerPosition = {x: anchor.x + x * 2, y: anchor.y};
        const furnacePosition = {x: minerPosition.x, y: minerPosition.y + 2};
        const minerEntity = await bot.placeEntity(minerName, minerPosition, 4); // place down/south
        const furnaceEntity = await bot.placeEntity(furnaceName, furnacePosition, 0); // place up/north but doesnt matter here
        const minerFurnace: StarterMinerFurnace = {
            minerPosition: minerEntity.position,
            furnacePosition: furnaceEntity.position,
            minerType: minerName,
            furnaceType: furnaceName,
            plateName: data.plateName,
            oreName: data.oreName
        }
        store.commit("addStarterMinerFurnace", minerFurnace)
        minerFurnaces.push(minerFurnace)
    }
    return minerFurnaces
}

taskRunnerByType[TASK_TYPE] = executeThisTask

export async function createBuildStarterMinerFurnaceTask(store: Store<State>, oreName: string, plateName: string, minerSmelterCount: number): Promise<Task> {
    const data: TaskData = {
        minerSmelterCount,
        oreName,
        plateName,
    }
    return createTask(TASK_TYPE, `Starter Miner/Furnace for ${oreName} x ${minerSmelterCount}`, data)
}