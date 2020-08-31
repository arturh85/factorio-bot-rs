import {FactorioBot} from "@/factorio-bot/bot";
import {Store} from "vuex";
import {State} from "@/store";
import {createTask, executeTask, registerTaskRunner, Task, TaskStatus, updateTaskStatus} from "@/factorio-bot/task";
import {Entities, InventoryType, StarterMinerChest} from "@/factorio-bot/types";
import {createCraftTask} from "@/factorio-bot/tasks/craft-task";

const TASK_TYPE = 'build-starter-miner-chest'
const chestName = Entities.ironChest;
const minerName = Entities.burnerMiningDrill;

type TaskData = {
    oreName: string,
    minerChestCount: number,
}

async function executeThisTask(store: Store<State>, bots: FactorioBot[], task: Task): Promise<StarterMinerChest[]> {
    const data: TaskData = task.data as TaskData
    if (bots.length === 0) {
        throw new Error("no bots?")
    }
    // sort by already has correct item
    // bots.sort(sortBotsByInventory([minerName, furnaceName]))
    const bot = bots[0]
    const botMiners = bot.mainInventory(minerName)
    const subtasks: Task[] = []
    if (botMiners < data.minerChestCount) {
        const subtask = await createCraftTask(store, minerName, data.minerChestCount, false)
        store.commit('addSubTask', {id: task.id, task: subtask})
        subtasks.push(subtask)
    }
    const botChests = bot.mainInventory(chestName)
    if (botChests < data.minerChestCount) {
        const subtask = await createCraftTask(store, chestName, data.minerChestCount, false)
        store.commit('addSubTask', {id: task.id, task: subtask})
        subtasks.push(subtask)
    }
    if (subtasks) {
        store.commit('updateTask', updateTaskStatus(task, TaskStatus.WAITING));
        for (const subTask of subtasks) {
            await executeTask(store, bots, subTask)
        }
        store.commit('updateTask', updateTaskStatus(task, TaskStatus.STARTED));
    }
    const excludePositions = Object.keys(store.state.players).map(key => store.state.players[key].position)
    const oreFieldTopLeft = await bot.findNearestRect(
        {x: 0, y: 0},
        data.oreName,
        2 * data.minerChestCount,
        2,
        excludePositions
    );
    if (!oreFieldTopLeft) {
        store.commit('updateTask', updateTaskStatus(task, TaskStatus.FAILED));
        throw new Error(`failed to find ore field for ${data.oreName}`)
    }
    const anchor = {...oreFieldTopLeft};
    anchor.x = Math.floor(anchor.x);
    anchor.y = Math.floor(anchor.y);
    const minerChests = [];
    for (let x = 0; x < data.minerChestCount; x++) {
        const minerPosition = {x: anchor.x + x * 2, y: anchor.y};
        const chestPosition = {x: minerPosition.x, y: minerPosition.y + 1};
        try {
            const minerEntity = await bot.placeEntity(minerName, minerPosition, 4) // place down/south
            if (bot.mainInventory(Entities.coal) > 2) {
                await bot.insertToInventory(minerName, minerEntity.position, InventoryType.chest_or_fuel, Entities.coal, 2)
            }
            const chestEntity = await bot.placeEntity(chestName, chestPosition, 0) // place up/north but doesnt matter here
            const minerChest: StarterMinerChest = {
                minerPosition: minerEntity.position,
                chestPosition: chestEntity.position,
                minerType: minerName,
                chestType: chestName,
                oreName: data.oreName
            }
            store.commit("addStarterMinerChest", minerChest)
            minerChests.push(minerChest)
        } catch(err) {
            console.warn("failed to place starter miner furnace", err)
        }
    }
    return minerChests
}

registerTaskRunner(TASK_TYPE, executeThisTask)

export async function createBuildStarterMinerChestTask(store: Store<State>, oreName: string, minerChestCount: number): Promise<Task> {
    const data: TaskData = {
        minerChestCount,
        oreName,
    }
    return createTask(TASK_TYPE, `Starter Miner/Chest for ${oreName} x ${minerChestCount}`, data)
}