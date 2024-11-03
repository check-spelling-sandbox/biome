// Examples which are not reported by the rule (false negative cases)

function f (x) {
    if (x < 0) {
        while(true) {
            return 0;
        }
    } else {
        return x;
    }
}

function f (x) {
    if (x > 0 && x < 5) {
        switch (x) {
            case 0:
                f(); // fallthrough
            default:
                return 0;
        }
    } else {
        return x;
    }
}


function f (x) {
    if (x > 0 && x < 5) {
        try {
            g(x);
        } catch {
            g(0);
        } finally {
            return 0;
        }
    } else {
        return x;
    }
}


function f (x) {
    if (x > 0 && x < 5) {
        try {
            g(x);
            return x;
        } catch {
            return 0;
        }
    } else {
        return x;
    }
}
