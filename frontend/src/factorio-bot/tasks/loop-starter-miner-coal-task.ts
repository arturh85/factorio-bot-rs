import {FactorioBot} from "@/factorio-bot/bot";
import {Store} from "vuex";
import {State} from "@/store";
import {createTask, executeTask, Task, taskRunnerByType, TaskStatus, updateTaskStatus} from "@/factorio-bot/task";
import {FactorioApi} from "@/factorio-bot/restApi";
import {Entities, InventoryType, RequestEntity} from "@/factorio-bot/types";
import {createGatherTask} from "@/factorio-bot/tasks/gather-task";
import {sleep} from "@/factorio-bot/util";

const TASK_TYPE = 'loop-starter-miner-coal'

type TaskData = {
    coalCount: number,
}

async function executeThisTask(store: Store<State>, bots: FactorioBot[], task: Task): Promise<void> {
    const data: TaskData = task.data as TaskData
    if (bots.length === 0) {
        return
    }
    // bots.sort(sortBotsByInventory([data.name]))
    const bot = bots[0]
    let remaining = data.coalCount
    while (remaining > 0) {
        store.commit('updateTask', updateTaskStatus(task, TaskStatus.STARTED));
        const inventoryContents = await FactorioApi.inventoryContentsAt((store.state.world.starterCoalLoops || [])
            .flatMap(coalLoop => {
                const entities: RequestEntity[] = [];
                entities.push({
                    name: coalLoop.minerType,
                    position: coalLoop.minerPosition,
                })
                return entities;
            }))
        // first get all coal
        for (const entity of inventoryContents) {
            if (entity.fuelInventory && entity.fuelInventory[Entities.coal]) {
                await bot.removeFromInventory(entity.name, entity.position, InventoryType.chest_or_fuel, Entities.coal, entity.fuelInventory[Entities.coal])
                remaining -= entity.fuelInventory[Entities.coal]
            }
        }
        break
        // if (remaining === 0) {
        //     break
        // }
        // store.commit('updateTask', updateTaskStatus(task, TaskStatus.SLEEPING));
        // await sleep(5000)
    }
}

taskRunnerByType[TASK_TYPE] = executeThisTask

export async function createLoopStarterMinerCoalTask(store: Store<State>, coalCount: number): Promise<Task> {
    const data: TaskData = {
        coalCount
    }
    return createTask(TASK_TYPE, `Loop Starter Miner Coal for coal x ${coalCount}`, data)
}