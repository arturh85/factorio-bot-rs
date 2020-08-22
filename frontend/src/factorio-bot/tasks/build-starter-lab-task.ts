import {FactorioBot} from "@/factorio-bot/bot";
import {Store} from "vuex";
import {State} from "@/store";
import {createTask, executeTask, Task, taskRunnerByType, TaskStatus, updateTaskStatus} from "@/factorio-bot/task";
import {entityRect, sortBotsByInventory} from "@/factorio-bot/util";
import {
    Direction,
    Entities,
    FactorioEntity,
    InventoryType,
    Position,
    Rect,
    StarterMinerFurnace
} from "@/factorio-bot/types";
import {createCraftTask} from "@/factorio-bot/tasks/craft-task";

const TASK_TYPE = 'build-starter-lab'
const labName = Entities.lab;

type TaskData = {
    labCount: number,
}

async function executeThisTask(store: Store<State>, bots: FactorioBot[], task: Task): Promise<FactorioEntity[]> {
    const data: TaskData = task.data as TaskData
    if (bots.length === 0) {
        throw new Error("no bots?")
    }
    // sort by already has correct item
    // bots.sort(sortBotsByInventory([minerName, furnaceName]))
    const offshorePumpPosition = store.state.world.starterOffshorePump
    if (!offshorePumpPosition) {
        throw new Error("lab requires offshore pump")
    }
    const bot = bots[0]
    if (bot.mainInventory(Entities.lab) < data.labCount) {
        const subtask = await createCraftTask(store, Entities.lab, data.labCount - bot.mainInventory(Entities.lab), false)
        store.commit('addSubTask', {id: task.id, task: subtask})
        store.commit('updateTask', updateTaskStatus(task, TaskStatus.WAITING));
        await executeTask(store, bots, subtask)
        store.commit('updateTask', updateTaskStatus(task, TaskStatus.STARTED));
    }
    const labs = []
    for (let labIndex = 0; labIndex < data.labCount; labIndex++) {
        const lab = await bot.placeEntity(
            Entities.lab,
            {
                x: offshorePumpPosition.x + 2 + labIndex * 3,
                y: offshorePumpPosition.y - 14,
            },
            Direction.north
        );
        store.commit('addStarterLab', lab.position)
        labs.push(lab)
    }
    return labs
}

taskRunnerByType[TASK_TYPE] = executeThisTask

export async function createBuildStarterLabTask(store: Store<State>, labCount: number): Promise<Task> {
    const data: TaskData = {
        labCount,
    }
    return createTask(TASK_TYPE, `Build Starter Lab x ${labCount}`, data)
}